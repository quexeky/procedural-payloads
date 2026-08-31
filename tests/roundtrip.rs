use procedural_payloads::read::{
    fields::ReadableFrameField,
    metadata::{ReadableMetadataField, UnCached},
    payload::ReadablePayload,
};
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

impl ReadableMetadataField<8> for Metadata {
    fn num_fields(&self) -> usize {
        64 - 8
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

impl ReadableFrameField<1> for Field {}

impl WritableMetadataField for Metadata {
    fn num_fields(&self) -> usize {
        64 - 8
    }

    fn write_to<W: embedded_io::Write>(self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_all(&self.random_data)
    }
}

#[test]
fn write_then_read_roundtrip() {
    let mut data = [0; 64];

    let metadata = Metadata {
        random_data: [3; 8],
    };

    let mut writer = WritablePayload::new(&mut data[..]).begin(metadata).unwrap();

    for i in 0..56u8 {
        writer.write_field(Field { data: i }).unwrap();
    }

    writer.finish().unwrap();

    let reader = ReadablePayload::<1, 8, Metadata, UnCached, Field, _>::new(data.as_slice());
    let reader = reader.load().unwrap();

    assert_eq!(*reader.metadata(), metadata);

    for (i, field) in reader.into_iter().enumerate() {
        assert_eq!(field.unwrap(), Field { data: i as u8 });
    }
}
