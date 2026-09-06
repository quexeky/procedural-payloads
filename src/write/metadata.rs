use crate::write::writeable::Writeable;

pub trait WritableMetadataField: Writeable {
    fn num_fields(&self) -> usize;
}

pub trait MetadataWriteState {}
pub struct Written {
    pub fields_remaining: usize,
}

impl Written {
    pub fn new(fields_remaining: usize) -> Self {
        Self { fields_remaining }
    }
}
impl MetadataWriteState for Written {}
pub struct NotWritten;
impl MetadataWriteState for NotWritten {}
