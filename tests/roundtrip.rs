#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use crc32fast::Hasher;
use embedded_io::{Read, Write};
use procedural_payloads::write::{
    fields::WritableFrameField, metadata::WritableMetadataField, payload::WritablePayload,
};
use procedural_payloads::{
    read::{
        crc_32_reader::Crc32Reader,
        metadata::{ReadableMetadataField, UnCached},
        payload::ReadablePayload,
    },
    write::crc_32_writer::Crc32Writer,
};
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

#[derive(PartialEq, Eq, Debug, Clone, Copy, IntoBytes, KnownLayout, Immutable, TryFromBytes)]
struct Metadata {
    random_data: [u8; 8],
}

impl ReadableMetadataField for Metadata {
    fn num_fields(&self) -> usize {
        64 - 8
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, IntoBytes, KnownLayout, Immutable, TryFromBytes)]
struct Field {
    data: u8,
}
impl WritableFrameField for Field {
    const SIZE: usize = 1;
    fn write_to<W: embedded_io::Write>(self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_all(&[self.data])
    }
}


impl WritableMetadataField for Metadata {
    fn num_fields(&self) -> usize {
        64 - 8
    }
}

#[test]
fn write_then_read_roundtrip() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();

    let metadata = Metadata {
        random_data: [3; 8],
    };

    let mut writer = WritablePayload::new(&mut data_slice)
        .begin(metadata)
        .unwrap();

    for i in 0..56u8 {
        writer.write_field(Field { data: i }).unwrap();
    }

    writer.finish().unwrap();
    let mut data_slice = data.as_slice();

    let reader = ReadablePayload::<Metadata, UnCached, Field, _>::new(&mut data_slice);
    let reader = reader.load().unwrap();

    assert_eq!(*reader.metadata(), metadata);

    for (i, field) in reader.into_iter().enumerate() {
        assert_eq!(field.unwrap(), Field { data: i as u8 });
    }
}

#[test]
fn write_then_read_roundtrip_with_crc() {
    let mut data = [0; 64];
    let mut data_slice = data.as_mut_slice();

    let metadata = Metadata {
        random_data: [3; 8],
    };

    let mut hasher = Crc32Writer::new(&mut data_slice, Hasher::new());

    let mut writer = WritablePayload::new(&mut hasher).begin(metadata).unwrap();

    for i in 0..56u8 {
        writer.write_field(Field { data: i }).unwrap();
    }

    writer.finish().unwrap();
    let written_crc = hasher.finish();
    assert_eq!(written_crc, crc32fast::hash(&data));

    let mut data_slice = data.as_slice();
    let mut hasher = Crc32Reader::new(&mut data_slice, Hasher::new());

    let reader = ReadablePayload::<Metadata, UnCached, Field, _>::new(&mut hasher);
    let reader = reader.load().unwrap();

    assert_eq!(*reader.metadata(), metadata);

    for (i, field) in reader.into_iter().enumerate() {
        assert_eq!(field.unwrap(), Field { data: i as u8 });
    }
    let read_crc = hasher.finish();
    assert_eq!(written_crc, read_crc);
}

#[test]
fn write_then_read_crcs_match_known_vector() {
    let mut data = [0u8; 9];
    let mut data_slice = data.as_mut_slice();
    let mut writer = Crc32Writer::new(&mut data_slice, Hasher::new());

    writer.write_all(b"123456789").unwrap();
    let written_crc = writer.finish();
    assert_eq!(written_crc, 0xCBF43926);

    let mut data_slice = data.as_slice();
    let mut reader = Crc32Reader::new(&mut data_slice, Hasher::new());

    let mut out = [0u8; 9];
    reader.read_exact(&mut out).unwrap();
    let read_crc = reader.finish();
    assert_eq!(read_crc, 0xCBF43926);
    assert_eq!(written_crc, read_crc);
}
