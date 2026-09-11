use core::ops::Deref;

use crate::read::readable::Readable;

pub trait ReadableMetadataField: Readable {
    fn num_fields(&self) -> usize;
}

pub trait MetadataState {}
pub struct Cached<M: ReadableMetadataField> {
    pub metadata: M,
}
impl<M: ReadableMetadataField> MetadataState for Cached<M> {}
pub struct UnCached;
impl MetadataState for UnCached {}

// --- Cached impls --- //

impl<M: ReadableMetadataField> Deref for Cached<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

// --- UnCached impls --- //
