use procedural_payloads::write::{
    fields::WritableFrameField, metadata::WritableMetadataField, payload::WritablePayload,
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
impl WritableFrameField<1> for Field {
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
    let mut payload = WritablePayload::new(&mut data[..])
        .begin(metadata)
        .expect("Failed to begin with metadata");

    for field in fields {
        payload.write_field(field).expect("Failed to write new field");
    }
    assert_eq!(data, expected_data);
}
