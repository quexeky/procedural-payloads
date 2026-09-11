use crate::read::{
    error::ReadError,
    fields::{FieldIterator, ReadableFrameField},
    metadata::{Cached, MetadataCache, MetadataState, ReadableMetadataField, UnCached},
};
use core::marker::PhantomData;
use embedded_io::Read;

pub struct ReadablePayload<
    M: ReadableMetadataField,
    S: MetadataState,
    T: ReadableFrameField,
    R: Read,
> {
    metadata: MetadataCache<S, M>,
    _field_iterator_marker: PhantomData<T>,
    reader: R,
}

//--- Cache impls ---//
impl<'a, M: ReadableMetadataField, T: ReadableFrameField, R: Read>
    ReadablePayload<M, Cached, T, R>
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

impl<'a, M: ReadableMetadataField, T: ReadableFrameField, R: Read> IntoIterator
    for ReadablePayload<M, Cached, T, R>
where
    [(); T::SIZE]:,
{
    type Item = Result<T, ReadError<R::Error>>;

    type IntoIter = FieldIterator<T, R>;

    fn into_iter(self) -> Self::IntoIter {
        FieldIterator::new(self.metadata.num_fields(), self.reader)
    }
}

// --- UnCached impls --- //
impl<'a, M: ReadableMetadataField, T: ReadableFrameField, R: Read>
    ReadablePayload<M, UnCached, T, R>
{
    pub fn new(reader: R) -> Self {
        Self {
            metadata: MetadataCache::new(),
            _field_iterator_marker: PhantomData,
            reader,
        }
    }
    pub fn load(mut self) -> Result<ReadablePayload<M, Cached, T, R>, ReadError<R::Error>>
    where
        [(); M::SIZE]:,
    {
        let metadata = self.metadata.load(&mut self.reader)?;
        Ok(ReadablePayload {
            metadata,
            _field_iterator_marker: PhantomData,
            reader: self.reader,
        })
    }
}
