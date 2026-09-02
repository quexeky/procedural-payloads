use crate::read::{
    error::Error,
    fields::{FieldIterator, ReadableFrameField},
    metadata::{Cached, MetadataCache, MetadataState, ReadableMetadataField, UnCached},
};
use core::marker::PhantomData;
use embedded_io::{Read, ReadExactError};

pub struct ReadablePayload<
    'a,
    const FIELD_SIZE: usize,
    const METADATA_SIZE: usize,
    M: ReadableMetadataField<METADATA_SIZE>,
    S: MetadataState,
    T: ReadableFrameField<FIELD_SIZE>,
    R: Read,
> {
    metadata: MetadataCache<METADATA_SIZE, S, M>,
    _field_iterator_marker: PhantomData<T>,
    reader: &'a mut R,
}

//--- Cache impls ---//
impl<
    'a,
    const FIELD_SIZE: usize,
    const METADATA_SIZE: usize,
    M: ReadableMetadataField<METADATA_SIZE>,
    T: ReadableFrameField<FIELD_SIZE>,
    R: Read,
> ReadablePayload<'a, FIELD_SIZE, METADATA_SIZE, M, Cached, T, R>
{
    pub fn from_metadata(reader: &'a mut R, metadata: M) -> Self {
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
    'a,
    const FIELD_SIZE: usize,
    const METADATA_SIZE: usize,
    M: ReadableMetadataField<METADATA_SIZE>,
    T: ReadableFrameField<FIELD_SIZE>,
    R: Read,
> IntoIterator for ReadablePayload<'a, FIELD_SIZE, METADATA_SIZE, M, Cached, T, R>
{
    type Item = Result<T, Error<R::Error, <T as TryFrom<[u8; FIELD_SIZE]>>::Error>>;

    type IntoIter = FieldIterator<'a, FIELD_SIZE, T, R>;

    fn into_iter(self) -> Self::IntoIter {
        FieldIterator::new(self.metadata.num_fields(), self.reader)
    }
}

// --- UnCached impls --- //
impl<
    'a,
    const FIELD_SIZE: usize,
    const METADATA_SIZE: usize,
    M: ReadableMetadataField<METADATA_SIZE>,
    T: ReadableFrameField<FIELD_SIZE>,
    R: Read,
> ReadablePayload<'a, FIELD_SIZE, METADATA_SIZE, M, UnCached, T, R>
{
    pub fn new(reader: &'a mut R) -> Self {
        Self {
            metadata: MetadataCache::new(),
            _field_iterator_marker: PhantomData,
            reader,
        }
    }
    pub fn load(
        mut self,
    ) -> Result<
        ReadablePayload<'a, FIELD_SIZE, METADATA_SIZE, M, Cached, T, R>,
        ReadExactError<R::Error>,
    > {
        let metadata = self.metadata.load(&mut self.reader)?;
        Ok(ReadablePayload {
            metadata,
            _field_iterator_marker: PhantomData,
            reader: self.reader,
        })
    }
}
