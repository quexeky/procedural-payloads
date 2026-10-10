# procedural-payloads

A `no_std` building block for payloads made of a fixed-size metadata prefix followed by a declared number of fixed-size fields. It supports incremental reads and writes, so embedded devices can stream telemetry without buffering an entire payload or allocating.

This crate is **not a complete serial protocol implementation**: framing, synchronization bytes, request/subscription messages, and device discovery belong to the transport or protocol built around it. It is intended to help build the kind of pub/sub debugging link described in [Designing a Serial communication protocol](https://blog.quexeky.dev/posts/designing-a-serial-communication-protocol--2026-08-19) and the [implementation follow-up](https://blog.quexeky.dev/posts/implementing-a-custom-serial-protocol-part-2--2026-08-27).

## Features

- Allocation-free metadata and field streaming.
- Synchronous APIs over [`embedded-io`](https://docs.rs/embedded-io/).
- Optional Embassy-compatible async APIs over [`embedded-io-async`](https://docs.rs/embedded-io-async/).
- CRC-32 reader and writer adapters for checksumming a stream.
- Type-state APIs that distinguish payloads before and after metadata is read or written.

## Toolchain

The crate currently uses Rust's `generic_const_exprs` feature to size buffers from field types, so it requires a nightly Rust toolchain.

The async APIs are optional and are enabled with the `async` feature:

```toml
[dependencies]
procedural-payloads = { version = "0.1", features = ["async"] }
```

## Async telemetry with Embassy

Embassy UART receive and transmit halves can be passed directly when they implement `embedded_io_async::Read` and `embedded_io_async::Write`. The example in [`examples/embassy.rs`](examples/embassy.rs) shows the read-side pattern; create your UART with the board's Embassy HAL and pass its split streams to the async function.

Define the payload metadata and field as zerocopy types. The metadata tells the reader how many fields follow it:

```rust
use procedural_payloads::{
    client::AsyncClient,
    read::{error::ReadError, metadata::ReadableMetadataField},
};
use embedded_io_async::{Read, Write};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

#[repr(C)]
#[derive(IntoBytes, Immutable, KnownLayout, TryFromBytes)]
struct TelemetryMetadata {
    sample_count: u8,
    stream_id: u8,
}

impl ReadableMetadataField for TelemetryMetadata {
    fn num_fields(&self) -> usize {
        usize::from(self.sample_count)
    }
}

#[repr(C)]
#[derive(Clone, Copy, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
struct MotorSample {
    position: i16,
    current_ma: u16,
}

async fn receive<R: Read, W: Write>(
    uart_rx: R,
    uart_tx: W,
) -> Result<(), ReadError<R::Error>> {
    let mut client = AsyncClient::new(uart_rx, uart_tx);
    let payload = client.read_payload::<TelemetryMetadata, MotorSample>().await?;
    let mut samples = payload.into_fields();

    while let Some(sample) = samples.next_field().await {
        let sample = sample?;
        // Inspect or forward each sample as it arrives.
        let _position = sample.position;
    }

    // Ensures the complete declared payload was consumed before reusing the stream.
    let _uart_rx = samples.finish()?;
    Ok(())
}
```

Writing uses the same metadata-defined field count:

```rust
use procedural_payloads::{client::AsyncClient, write::metadata::WritableMetadataField};
use zerocopy::{Immutable, IntoBytes};

impl WritableMetadataField for TelemetryMetadata {
    fn num_fields(&self) -> usize {
        usize::from(self.sample_count)
    }
}

async fn transmit<W: embedded_io_async::Write>(
    uart_tx: W,
    metadata: TelemetryMetadata,
    samples: &[MotorSample],
) -> Result<(), procedural_payloads::write::error::WriteError<W::Error>> {
    let mut client = AsyncClient::new((), uart_tx);
    let mut payload = client.write_payload::<TelemetryMetadata, MotorSample>(metadata).await?;
    for sample in samples.iter().copied() {
        payload.write_field(sample).await?;
    }
    let _uart_tx = payload.finish().await?;
    Ok(())
}
```

`write_payload` requires metadata and fields to implement zerocopy's `IntoBytes + Immutable`; `WritableMetadataField` and `WritableFrameField` provide the payload-specific field counts and sizes. A payload writer rejects attempts to write more or fewer fields than metadata declared. The bytes are copied from the Rust type's memory representation without byte-order conversion; choose explicit wire endianness (for example, zerocopy's endian-aware integer types) when defining fields that cross architectures.

Async `read_exact`/`write_all` operations may make partial progress if their futures are cancelled. Do not cancel an in-flight field operation and then resume parsing the same stream as if no bytes moved; the underlying Embassy driver determines the details of cancellation behavior.

## CRC-32

Wrap the stream at the payload boundary to include both metadata and fields in the checksum. After reading all declared fields, call `AsyncCrc32Reader::validate().await` to read and verify the four-byte big-endian checksum trailer. For output, `AsyncCrc32Writer::finish().await` appends that trailer and flushes it. The synchronous equivalents are in `read::crc_32_reader` and `write::crc_32_writer`.

## Build and test

```sh
cargo +nightly test --no-default-features --all-targets
cargo +nightly test --all-features --all-targets
cargo +nightly test --all-features --doc
```
