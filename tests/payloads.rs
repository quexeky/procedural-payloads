use core::slice;

use procedural_payloads::{fields::FrameField, metadata::MetadataField, payload::Payload};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Metadata {
    random_data: [u8; 8],
}

impl From<[u8; 8]> for Metadata {
    fn from(value: [u8; 8]) -> Self {
        Self { random_data: value }
    }
}
impl AsRef<[u8]> for Metadata {
    fn as_ref(&self) -> &[u8] {
        self.random_data.as_slice()
    }
}
#[derive(PartialEq, Eq, Debug)]
struct Field {
    data: u8,
}
impl FrameField<1> for Field {}

impl From<[u8; 1]> for Field {
    fn from(value: [u8; 1]) -> Self {
        Field { data: value[0] }
    }
}
impl AsRef<[u8]> for Field {
    fn as_ref(&self) -> &[u8] {
        slice::from_ref(&self.data)
    }
}

impl MetadataField<8> for Metadata {
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
    let payload = Payload::new(data.as_slice()).load().unwrap();
    let metadata: Metadata = *payload.metadata();

    assert_eq!(metadata, Metadata::from(metadata_chunk));

    let iter = payload.into_iter();

    for (i, field) in iter.enumerate() {
        assert_eq!(Field { data: data[i + 8] }, field.unwrap());
    }
}
