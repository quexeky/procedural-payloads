use crate::read::{
    fields::{FieldIterator, ReadableFrameField},
    metadata::{Cached, MetadataCache, ReadableMetadataField, MetadataState, UnCached},
};

use core::marker::PhantomData;
use embedded_io::{Read, ReadExactError};

pub struct ReadablePayload<
    const FIELD_SIZE: usize,
    const METADATA_SIZE: usize,
    M: ReadableMetadataField<METADATA_SIZE>,
    S: MetadataState,
    T: ReadableFrameField<FIELD_SIZE>,
    R: Read,
> {
    metadata: MetadataCache<METADATA_SIZE, S, M>,
    _field_iterator_marker: PhantomData<T>,
    reader: R,
}

//--- Cache impls ---//
impl<
    const FIELD_SIZE: usize,
    const METADATA_SIZE: usize,
    M: ReadableMetadataField<METADATA_SIZE>,
    T: ReadableFrameField<FIELD_SIZE>,
    R: Read,
> ReadablePayload<FIELD_SIZE, METADATA_SIZE, M, Cached, T, R>
{
    pub fn from_metadata(reader: R, metadata: M) -> Self {
        Self {
            metadata: MetadataCache::new_init(metadata),
            _field_iterator_marker: PhantomData,
            reader,
        }
    }
    pub fn metadata(&self) -> &M {
        &self.metadata
    }
}

impl<
    const FIELD_SIZE: usize,
    const METADATA_SIZE: usize,
    M: ReadableMetadataField<METADATA_SIZE>,
    T: ReadableFrameField<FIELD_SIZE>,
    R: Read,
> IntoIterator for ReadablePayload<FIELD_SIZE, METADATA_SIZE, M, Cached, T, R>
{
    type Item = Result<T, ReadExactError<R::Error>>;

    type IntoIter = FieldIterator<FIELD_SIZE, T, R>;

    fn into_iter(self) -> Self::IntoIter {
        FieldIterator::new(self.metadata.num_fields(), self.reader)
    }
}

// --- UnCached impls --- //
impl<
    const FIELD_SIZE: usize,
    const METADATA_SIZE: usize,
    M: ReadableMetadataField<METADATA_SIZE>,
    T: ReadableFrameField<FIELD_SIZE>,
    R: Read,
> ReadablePayload<FIELD_SIZE, METADATA_SIZE, M, UnCached, T, R>
{
    pub fn new(reader: R) -> Self {
        Self {
            metadata: MetadataCache::new(),
            _field_iterator_marker: PhantomData,
            reader,
        }
    }
    pub fn load(
        mut self,
    ) -> Result<ReadablePayload<FIELD_SIZE, METADATA_SIZE, M, Cached, T, R>, ReadExactError<R::Error>> {
        let metadata = self.metadata.load(&mut self.reader)?;
        Ok(ReadablePayload {
            metadata,
            _field_iterator_marker: PhantomData,
            reader: self.reader,
        })
    }
}
