#![cfg(feature = "async")]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use core::future::Future;
use std::task::{Context, Poll, Waker};

use embedded_io_async::{Read, Write};
use procedural_payloads::{
    client::AsyncClient,
    read::{
        asynchronous::{AsyncCrc32Reader, AsyncReadablePayload},
        error::ReadError,
        metadata::{Cached, ReadableMetadataField, UnCached},
    },
    write::{
        asynchronous::{AsyncCrc32Writer, AsyncWritablePayload},
        error::WriteError,
        metadata::{NotWritten, WritableMetadataField},
    },
};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

fn block_on<F: Future>(future: F) -> F::Output {
    let mut context = Context::from_waker(Waker::noop());
    let mut future = std::pin::pin!(future);

    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("slice-backed async IO should be immediately ready"),
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
struct Metadata {
    field_count: u8,
    stream_id: u8,
}

impl ReadableMetadataField for Metadata {
    fn num_fields(&self) -> usize {
        usize::from(self.field_count)
    }
}

impl WritableMetadataField for Metadata {
    fn num_fields(&self) -> usize {
        usize::from(self.field_count)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
struct Sample(u8);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
struct ValidatedByte(bool);

#[test]
fn async_client_reads_metadata_and_exactly_the_declared_fields() {
    block_on(async {
        let input = [2, 9, 0x31, 0x32, 0xff];
        let mut source = &input[..];
        let mut sink: [u8; 0] = [];
        {
            let mut client = AsyncClient::new(&mut source, &mut sink[..]);

            let payload = client.read_payload::<Metadata, Sample>().await.unwrap();
            assert_eq!(
                *payload.metadata(),
                Metadata {
                    field_count: 2,
                    stream_id: 9,
                }
            );

            let mut fields = payload.into_fields();
            assert_eq!(fields.next_field().await.unwrap().unwrap(), Sample(0x31));
            assert_eq!(fields.next_field().await.unwrap().unwrap(), Sample(0x32));
            assert!(fields.next_field().await.is_none());
            assert!(fields.finish().is_ok());
        }
        assert_eq!(source, &[0xff]);
    });
}

#[test]
fn async_payload_load_reports_short_metadata() {
    block_on(async {
        let input = [1];
        let mut source = &input[..];
        let error = match AsyncReadablePayload::<UnCached, Sample, _>::new(&mut source)
            .load::<Metadata>()
            .await
        {
            Err(error) => error,
            Ok(_) => panic!("short metadata must fail"),
        };

        assert!(matches!(error, ReadError::ReadExact(_)));
    });
}

#[repr(transparent)]
#[derive(Immutable, KnownLayout, TryFromBytes)]
struct CheckedMetadata(bool);

impl ReadableMetadataField for CheckedMetadata {
    fn num_fields(&self) -> usize {
        usize::from(self.0)
    }
}

#[test]
fn async_payload_load_rejects_invalid_metadata_bytes() {
    block_on(async {
        let input = [2];
        let mut source = &input[..];
        let result = AsyncReadablePayload::<UnCached, Sample, _>::new(&mut source)
            .load::<CheckedMetadata>()
            .await;

        assert!(matches!(result, Err(ReadError::InvalidCast)));
    });
}

#[test]
fn async_field_reader_reports_invalid_bytes_and_incomplete_finish() {
    block_on(async {
        let input = [2];
        let mut source = &input[..];
        let mut fields = procedural_payloads::read::asynchronous::AsyncFieldReader::<
            ValidatedByte,
            _,
        >::new(1, &mut source);

        assert!(matches!(
            fields.next_field().await,
            Some(Err(ReadError::InvalidCast))
        ));
        assert!(fields.next_field().await.is_none());
        assert!(matches!(fields.finish(), Err(ReadError::InsufficientData)));
    });
}

#[test]
fn async_field_reader_rejects_finishing_before_all_fields_are_read() {
    block_on(async {
        let input = [1, 2];
        let mut source = &input[..];
        let mut fields =
            procedural_payloads::read::asynchronous::AsyncFieldReader::<Sample, _>::new(
                2,
                &mut source,
            );

        assert_eq!(fields.next_field().await.unwrap().unwrap(), Sample(1));
        assert!(matches!(fields.finish(), Err(ReadError::InsufficientData)));

        let mut source = &[][..];
        let mut fields =
            procedural_payloads::read::asynchronous::AsyncFieldReader::<Sample, _>::new(
                1,
                &mut source,
            );
        assert!(matches!(
            fields.next_field().await,
            Some(Err(ReadError::ReadExact(_)))
        ));
        assert!(matches!(fields.finish(), Err(ReadError::InsufficientData)));
    });
}

#[test]
fn async_zero_field_payload_finishes_without_consuming_following_data() {
    block_on(async {
        let input = [0, 7, 0xaa];
        let mut source = &input[..];
        let payload = AsyncReadablePayload::<UnCached, Sample, _>::new(&mut source)
            .load::<Metadata>()
            .await
            .unwrap();

        assert_eq!(payload.metadata().stream_id, 7);
        assert!(payload.into_fields().finish().is_ok());
        assert_eq!(source, &[0xaa]);
    });
}

#[test]
fn async_payload_can_start_from_predecoded_metadata() {
    block_on(async {
        let input = [0x41, 0x42];
        let mut source = &input[..];
        let payload = AsyncReadablePayload::<Cached<Metadata>, Sample, _>::from_metadata(
            &mut source,
            Metadata {
                field_count: 2,
                stream_id: 4,
            },
        );
        let mut fields = payload.into_fields();

        assert_eq!(fields.next_field().await.unwrap().unwrap(), Sample(0x41));
        assert_eq!(fields.next_field().await.unwrap().unwrap(), Sample(0x42));
        assert!(fields.finish().is_ok());
    });
}

#[test]
fn async_client_can_borrow_and_return_its_streams() {
    let mut client = AsyncClient::new(1, 2);
    {
        let (reader, writer) = client.split();
        *reader = 3;
        *writer = 4;
    }
    assert_eq!(client.into_parts(), (3, 4));
}

#[test]
fn async_writer_writes_metadata_and_fields_and_enforces_field_count() {
    block_on(async {
        let mut output = [0; 4];
        let mut empty_source: &[u8] = &[];
        {
            let mut client = AsyncClient::new(&mut empty_source, &mut output[..]);
            let mut payload = client
                .write_payload::<Metadata, Sample>(Metadata {
                    field_count: 2,
                    stream_id: 7,
                })
                .await
                .unwrap();

            assert_eq!(payload.fields_remaining(), 2);
            payload.write_field(Sample(0x11)).await.unwrap();
            payload.write_field(Sample(0x22)).await.unwrap();
            assert!(matches!(
                payload.write_field(Sample(0x33)).await,
                Err(WriteError::ExcessData)
            ));
            assert!(payload.finish().await.unwrap().is_empty());
        }
        assert_eq!(output, [2, 7, 0x11, 0x22]);
    });
}

#[test]
fn async_writer_finishes_zero_field_payloads() {
    block_on(async {
        let mut output = [0; 2];
        let mut empty_source: &[u8] = &[];
        {
            let mut client = AsyncClient::new(&mut empty_source, &mut output[..]);
            let payload = client
                .write_payload::<Metadata, Sample>(Metadata {
                    field_count: 0,
                    stream_id: 5,
                })
                .await
                .unwrap();

            assert_eq!(payload.fields_remaining(), 0);
            assert!(payload.finish().await.unwrap().is_empty());
        }
        assert_eq!(output, [0, 5]);
    });
}

#[test]
fn async_writer_finish_rejects_missing_fields() {
    block_on(async {
        let mut output = [0; 4];
        let writer: &mut [u8] = &mut output;
        let payload = AsyncWritablePayload::<Metadata, NotWritten, Sample, _>::new(writer)
            .begin(Metadata {
                field_count: 1,
                stream_id: 0,
            })
            .await
            .unwrap();

        assert!(matches!(
            payload.finish().await,
            Err(WriteError::InsufficientDataWritten)
        ));
    });
}

#[test]
fn async_writer_propagates_underlying_io_failures() {
    block_on(async {
        let mut metadata_buffer = [0; 1];
        let writer: &mut [u8] = &mut metadata_buffer;
        let result = AsyncWritablePayload::<Metadata, NotWritten, Sample, _>::new(writer)
            .begin(Metadata {
                field_count: 1,
                stream_id: 0,
            })
            .await;
        assert!(matches!(result, Err(WriteError::Other(_))));

        let mut field_buffer = [0; 2];
        let writer: &mut [u8] = &mut field_buffer;
        let mut payload = AsyncWritablePayload::<Metadata, NotWritten, Sample, _>::new(writer)
            .begin(Metadata {
                field_count: 1,
                stream_id: 0,
            })
            .await
            .unwrap();
        assert!(matches!(
            payload.write_field(Sample(1)).await,
            Err(WriteError::Other(_))
        ));
        assert_eq!(payload.fields_remaining(), 1);
    });
}

#[test]
fn async_writer_rejects_payloads_exceeding_the_length_limit() {
    #[repr(transparent)]
    #[derive(IntoBytes, Immutable)]
    struct TooManyFields(u8);
    impl WritableMetadataField for TooManyFields {
        fn num_fields(&self) -> usize {
            usize::MAX
        }
    }

    block_on(async {
        let mut output = [0; 1];
        let writer: &mut [u8] = &mut output;
        let result = AsyncWritablePayload::<TooManyFields, NotWritten, Sample, _>::new(writer)
            .begin(TooManyFields(0))
            .await;

        assert!(matches!(result, Err(WriteError::TooMuchPlannedData)));
    });
}

#[test]
fn async_crc32_writer_appends_big_endian_checksum() {
    block_on(async {
        let mut output = [0; 13];
        let mut writer = AsyncCrc32Writer::new(&mut output[..]);
        writer.write_all(b"123456789").await.unwrap();

        assert_eq!(writer.checksum(), 0xcbf4_3926);
        assert!(writer.finish().await.unwrap().is_empty());
        assert_eq!(&output, b"123456789\xcb\xf4\x39\x26");
    });
}

#[test]
fn async_client_roundtrips_payloads_with_crc_wrappers() {
    block_on(async {
        let mut output = [0; 8];
        let mut empty_source: &[u8] = &[];
        let crc_writer = AsyncCrc32Writer::new(&mut output[..]);
        let mut client = AsyncClient::new(&mut empty_source, crc_writer);
        {
            let mut payload = client
                .write_payload::<Metadata, Sample>(Metadata {
                    field_count: 2,
                    stream_id: 7,
                })
                .await
                .unwrap();
            payload.write_field(Sample(0x11)).await.unwrap();
            payload.write_field(Sample(0x22)).await.unwrap();
            let _ = payload.finish().await.unwrap();
        }
        let (_, crc_writer) = client.into_parts();
        crc_writer.finish().await.unwrap();
        assert_eq!(&output[4..], &crc32fast::hash(&output[..4]).to_be_bytes());

        let mut source = &output[..];
        let mut sink: [u8; 0] = [];
        let crc_reader = AsyncCrc32Reader::new(&mut source);
        let mut client = AsyncClient::new(crc_reader, &mut sink[..]);
        {
            let payload = client.read_payload::<Metadata, Sample>().await.unwrap();
            let mut fields = payload.into_fields();
            assert_eq!(fields.next_field().await.unwrap().unwrap(), Sample(0x11));
            assert_eq!(fields.next_field().await.unwrap().unwrap(), Sample(0x22));
            let _ = fields.finish().unwrap();
        }
        let (crc_reader, _) = client.into_parts();
        crc_reader.validate().await.unwrap();
        assert!(source.is_empty());
    });
}

#[test]
fn async_crc32_reader_validates_checksum_and_detects_corruption() {
    block_on(async {
        let valid = *b"123456789\xcb\xf4\x39\x26";
        let mut source = &valid[..];
        let mut reader = AsyncCrc32Reader::new(&mut source);
        let mut payload = [0; 9];
        reader.read_exact(&mut payload).await.unwrap();
        assert_eq!(&payload, b"123456789");
        assert_eq!(reader.checksum(), 0xcbf4_3926);
        reader.validate().await.unwrap();

        let corrupted = *b"123456789\x00\x00\x00\x00";
        let mut source = &corrupted[..];
        let mut reader = AsyncCrc32Reader::new(&mut source);
        let mut payload = [0; 9];
        reader.read_exact(&mut payload).await.unwrap();
        assert!(matches!(
            reader.validate().await,
            Err(ReadError::InvalidCrc { .. })
        ));
    });
}

#[test]
fn async_crc32_reader_reports_a_truncated_trailer() {
    block_on(async {
        let input = [1, 2, 3];
        let mut source = &input[..];
        let reader = AsyncCrc32Reader::new(&mut source);

        assert!(matches!(
            reader.validate().await,
            Err(ReadError::ReadExact(_))
        ));
    });
}

#[test]
fn async_crc32_adapters_cover_only_transferred_bytes() {
    block_on(async {
        let mut short_output = [0; 4];
        let mut writer = AsyncCrc32Writer::new(&mut short_output[..]);
        assert!(writer.write_all(b"12345").await.is_err());
        assert_eq!(writer.checksum(), crc32fast::hash(b"1234"));
        assert!(writer.finish().await.is_err());

        let mut short_input = &b"1234"[..];
        let mut reader = AsyncCrc32Reader::new(&mut short_input);
        let mut bytes = [0; 8];
        assert_eq!(reader.read(&mut bytes).await.unwrap(), 4);
        assert_eq!(reader.checksum(), crc32fast::hash(b"1234"));
        let (remaining_input, hasher) = reader.into_inner();
        assert!(remaining_input.is_empty());
        assert_eq!(hasher.finalize(), crc32fast::hash(b"1234"));
    });
}

#[repr(transparent)]
#[derive(IntoBytes, Immutable)]
struct PlannedFields(u16);

impl WritableMetadataField for PlannedFields {
    fn num_fields(&self) -> usize {
        usize::from(self.0)
    }
}

#[test]
fn async_writer_enforces_the_payload_size_boundary() {
    block_on(async {
        let mut rejected_output = [0; 2];
        let result = AsyncWritablePayload::<PlannedFields, NotWritten, Sample, _>::new(
            &mut rejected_output[..],
        )
        .begin(PlannedFields(65_534))
        .await;
        assert!(matches!(result, Err(WriteError::TooMuchPlannedData)));
        assert_eq!(rejected_output, [0; 2]);

        let mut accepted_output = [0; 2];
        let payload = AsyncWritablePayload::<PlannedFields, NotWritten, Sample, _>::new(
            &mut accepted_output[..],
        )
        .begin(PlannedFields(65_533))
        .await
        .unwrap();
        assert_eq!(payload.fields_remaining(), 65_533);
    });
}
