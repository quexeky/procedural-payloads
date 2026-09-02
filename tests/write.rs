use crc32fast::Hasher;
use embedded_io::Write;
use procedural_payloads::write::{
    crc_32_writer::Crc32Writer, error::Error, fields::WritableFrameField,
    metadata::WritableMetadataField, payload::WritablePayload,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Metadata {
    random_data: [u8; 8],
}

impl From<[u8; 8]> for Metadata {
    fn from(value: [u8; 8]) -> Self {
        Self { random_data: value }
    }
}
#[derive(PartialEq, Eq, Debug)]
struct Field {
    data: u8,
}
impl WritableFrameField for Field {
    const SIZE: usize = 1;
    fn write_to<W: embedded_io::Write>(self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_all(&[self.data])
    }
}

impl From<[u8; 1]> for Field {
    fn from(value: [u8; 1]) -> Self {
        Field { data: value[0] }
    }
}

impl WritableMetadataField for Metadata {
    fn num_fields(&self) -> usize {
        64 - 8
    }

    fn write_to<W: embedded_io::Write>(self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_all(&self.random_data)
    }
}

#[test]
fn create_payload() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();
    let expected_data = [
        0, 0, 0, 0, 0, 0, 0, 0, //
        0, 1, 2, 3, 4, 5, 6, 7, //
        8, 9, 10, 11, 12, 13, 14, 15, //
        16, 17, 18, 19, 20, 21, 22, 23, //
        24, 25, 26, 27, 28, 29, 30, 31, //
        32, 33, 34, 35, 36, 37, 38, 39, //
        40, 41, 42, 43, 44, 45, 46, 47, //
        48, 49, 50, 51, 52, 53, 54, 55, //
    ];
    let fields = (0..56).map(|i| Field { data: i });
    let metadata = Metadata {
        random_data: [0; 8],
    };
    let mut payload = WritablePayload::new(&mut data_slice)
        .begin(metadata)
        .expect("Failed to begin with metadata");

    for field in fields {
        payload
            .write_field(field)
            .expect("Failed to write new field");
    }
    assert_eq!(data, expected_data);
}

struct EmptyMetadata {
    fields: usize,
}

impl WritableMetadataField for EmptyMetadata {
    fn num_fields(&self) -> usize {
        self.fields
    }

    fn write_to<W: embedded_io::Write>(self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_all(&[])
    }
}

#[test]
fn finish_is_ok_when_all_fields_are_written() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();
    let metadata = Metadata {
        random_data: [0; 8],
    };

    let mut payload = WritablePayload::<Metadata, _, Field, _>::new(&mut data_slice)
        .begin(metadata)
        .unwrap();

    for i in 0..56u8 {
        payload.write_field(Field { data: i }).unwrap();
    }

    payload.finish().unwrap();
}

#[test]
fn finish_errors_when_no_fields_have_been_written() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();
    let metadata = Metadata {
        random_data: [0; 8],
    };

    let payload = WritablePayload::<Metadata, _, Field, _>::new(&mut data_slice)
        .begin(metadata)
        .unwrap();

    let err = payload.finish().unwrap_err();
    assert!(matches!(err, Error::InsufficientDataWritten));
}

#[test]
fn finish_errors_when_some_fields_are_missing() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();
    let metadata = Metadata {
        random_data: [0; 8],
    };

    let mut payload = WritablePayload::<Metadata, _, Field, _>::new(&mut data_slice)
        .begin(metadata)
        .unwrap();

    for i in 0..10u8 {
        payload.write_field(Field { data: i }).unwrap();
    }

    let err = payload.finish().unwrap_err();
    assert!(matches!(err, Error::InsufficientDataWritten));
}

#[test]
fn write_field_errors_after_all_fields_are_written() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();
    let metadata = Metadata {
        random_data: [0; 8],
    };

    let mut payload = WritablePayload::<Metadata, _, Field, _>::new(&mut data_slice)
        .begin(metadata)
        .unwrap();

    for i in 0..56u8 {
        payload.write_field(Field { data: i }).unwrap();
    }

    let err = payload.write_field(Field { data: 56 }).unwrap_err();

    assert!(matches!(err, Error::ExcessData));
}

#[test]
fn begin_writes_metadata_before_any_fields() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();

    let _ = WritablePayload::<Metadata, _, Field, _>::new(&mut data_slice)
        .begin(Metadata {
            random_data: [9; 8],
        })
        .unwrap();

    assert_eq!(&data[..8], &[9; 8]);
    assert_eq!(&data[8..], &[0; 56]);
}

#[test]
fn metadata_and_fields_are_written_in_order() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();
    let metadata = Metadata {
        random_data: [0xAB; 8],
    };

    let mut payload = WritablePayload::<Metadata, _, Field, _>::new(&mut data_slice)
        .begin(metadata)
        .unwrap();

    for i in 0..56u8 {
        payload.write_field(Field { data: i }).unwrap();
    }

    payload.finish().unwrap();

    assert_eq!(&data[..8], &[0xAB; 8]);
    for (i, byte) in data[8..].iter().enumerate() {
        assert_eq!(*byte, i as u8);
    }
}

#[test]
fn begin_rejects_exactly_65536_planned_field_bytes() {
    let mut buf: [u8; 0] = [];
    let mut buf_slice = buf.as_mut_slice();

    let result = WritablePayload::<EmptyMetadata, _, Field, _>::new(&mut buf_slice)
        .begin(EmptyMetadata { fields: 65536 });

    assert!(matches!(result, Err(Error::TooMuchPlannedData)));
}

#[test]
fn begin_accepts_65535_planned_field_bytes() {
    let mut buf: [u8; 0] = [];
    let mut buf_slice = buf.as_mut_slice();

    let result = WritablePayload::<EmptyMetadata, _, Field, _>::new(&mut buf_slice)
        .begin(EmptyMetadata { fields: 65535 });

    assert!(result.is_ok());
}

#[test]
fn writer_forwards_data_unchanged() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();
    let metadata = Metadata {
        random_data: [0; 8],
    };
    let mut writer = Crc32Writer::new(&mut data_slice, Hasher::new());

    let mut payload = WritablePayload::<Metadata, _, Field, _>::new(&mut writer)
        .begin(metadata)
        .unwrap();

    for i in 0..56u8 {
        payload.write_field(Field { data: i }).unwrap();
    }
    payload.finish().unwrap();

    assert_eq!(writer.finish(), crc32fast::hash(&data));
}

#[test]
fn writer_crc_matches_known_vector() {
    let mut data = [0u8; 9];
    let mut data_slice = data.as_mut_slice();
    let mut writer = Crc32Writer::new(&mut data_slice, Hasher::new());

    writer.write_all(b"123456789").unwrap();

    assert_eq!(writer.finish(), 0xCBF43926);
}

#[test]
fn writer_crc_matches_chunked_writes() {
    let mut data = [0u8; 9];
    let mut data_slice = data.as_mut_slice();
    let mut writer = Crc32Writer::new(&mut data_slice, Hasher::new());

    for chunk in b"123456789".chunks(3) {
        let written = writer.write(chunk).unwrap();
        assert_eq!(written, chunk.len());
    }

    assert_eq!(writer.finish(), crc32fast::hash(b"123456789"));
}

#[test]
fn writer_forwards_bytes_unchanged() {
    let mut data = [0u8; 9];
    let mut data_slice = data.as_mut_slice();
    let mut writer = Crc32Writer::new(&mut data_slice, Hasher::new());

    writer.write_all(b"123456789").unwrap();

    assert_eq!(data, *b"123456789");
}

#[test]
fn writer_short_write_crc_covers_only_written_bytes() {
    let mut data = [0u8; 2];
    let mut data_slice = data.as_mut_slice();
    let mut writer = Crc32Writer::new(&mut data_slice, Hasher::new());

    let written = writer.write(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(written, 2);

    assert_eq!(writer.finish(), crc32fast::hash(&[1, 2]));
}
