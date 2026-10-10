#![feature(generic_const_exprs)]
#![allow(dead_code, incomplete_features)]

use embedded_io_async::{Read, Write};
use procedural_payloads::{
    client::AsyncClient,
    read::{error::ReadError, metadata::ReadableMetadataField},
};
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
#[derive(IntoBytes, Immutable, KnownLayout, TryFromBytes)]
struct MotorSample {
    position: i16,
    current_ma: u16,
}

async fn read_motor_samples<R, W>(receive: R, transmit: W) -> Result<(), ReadError<R::Error>>
where
    R: Read,
    W: Write,
{
    let mut client = AsyncClient::new(receive, transmit);
    let payload = client
        .read_payload::<TelemetryMetadata, MotorSample>()
        .await?;
    let mut samples = payload.into_fields();

    while let Some(sample) = samples.next_field().await {
        let sample = sample?;
        let _position = sample.position;
        let _current_ma = sample.current_ma;
    }

    let _receive_stream = samples.finish()?;
    Ok(())
}

fn main() {}
