use procedural_payloads::read::{
    fields::ReadableFrameField, metadata::ReadableMetadataField, payload::ReadablePayload,
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
impl ReadableFrameField<1> for Field {}

impl From<[u8; 1]> for Field {
    fn from(value: [u8; 1]) -> Self {
        Field { data: value[0] }
    }
}

impl ReadableMetadataField<8> for Metadata {
    fn num_fields(&self) -> usize {
        64 - 8
    }
}

#[test]
fn create_payload() {
    let data = [
        1, 2, 3, 4, 5, 6, 7, 8, //
        2, 2, 3, 4, 5, 6, 7, 8, //
        3, 2, 3, 4, 5, 6, 7, 8, //
        4, 2, 3, 4, 5, 6, 7, 8, //
        5, 2, 3, 4, 5, 6, 7, 8, //
        6, 2, 3, 4, 5, 6, 7, 8, //
        7, 2, 3, 4, 5, 6, 7, 8, //
        8, 2, 3, 4, 5, 6, 7, 8, //
    ];
    let metadata_chunk: [u8; 8] = data[0..8].try_into().unwrap();
    let payload = ReadablePayload::new(data.as_slice()).load().unwrap();
    let metadata: Metadata = *payload.metadata();

    assert_eq!(metadata, Metadata::from(metadata_chunk));

    let iter = payload.into_iter();

    for (i, field) in iter.enumerate() {
        assert_eq!(Field { data: data[i + 8] }, field.unwrap());
    }
}

use procedural_payloads::read::fields::FieldIterator;
use procedural_payloads::read::metadata::{Cached, UnCached};

#[test]
fn load_fails_when_metadata_is_short() {
    let short = [0u8; 7];

    let payload =
        ReadablePayload::<1, 8, Metadata, UnCached, Field, _>::new(short.as_slice());

    assert!(payload.load().is_err());
}

#[test]
fn iterator_reports_read_error_when_fields_are_missing() {
    let data = [0u8; 10];

    let payload =
        ReadablePayload::<1, 8, Metadata, UnCached, Field, _>::new(data.as_slice());
    let payload = payload.load().unwrap();

    let mut iter = payload.into_iter();

    assert!(matches!(iter.next(), Some(Ok(_))));
    assert!(matches!(iter.next(), Some(Ok(_))));
    assert!(matches!(iter.next(), Some(Err(_))));
}

#[test]
fn iterator_returns_exactly_metadata_field_count() {
    let data = [1u8; 64];

    let payload =
        ReadablePayload::<1, 8, Metadata, UnCached, Field, _>::new(data.as_slice());
    let payload = payload.load().unwrap();

    let mut count = 0;
    for field in payload.into_iter() {
        field.unwrap();
        count += 1;
    }

    assert_eq!(count, 56);
}

#[test]
fn create_payload_from_existing_metadata() {
    let mut field_data = [0u8; 56];
    for (i, byte) in field_data.iter_mut().enumerate() {
        *byte = i as u8;
    }

    let metadata = Metadata::from([7; 8]);
    let payload =
        ReadablePayload::<1, 8, Metadata, Cached, Field, _>::from_metadata(
            field_data.as_slice(),
            metadata,
        );

    assert_eq!(*payload.metadata(), Metadata::from([7; 8]));

    for (i, field) in payload.into_iter().enumerate() {
        assert_eq!(field.unwrap(), Field { data: i as u8 });
    }
}

#[test]
fn field_iterator_returns_none_when_no_fields_remain() {
    let data: [u8; 0] = [];

    let mut iter = FieldIterator::<1, Field, _>::new(0, data.as_slice());

    assert!(iter.next().is_none());
}

#[test]
fn field_iterator_reads_each_field() {
    let data = [1u8, 2u8];

    let mut iter = FieldIterator::<1, Field, _>::new(2, data.as_slice());

    assert_eq!(iter.next().unwrap().unwrap(), Field { data: 1 });
    assert_eq!(iter.next().unwrap().unwrap(), Field { data: 2 });
    assert!(iter.next().is_none());
}