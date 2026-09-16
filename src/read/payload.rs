use crate::read::{
    error::ReadError,
    fields::{FieldIterator, ReadableFrameField},
    metadata::{Cached, MetadataState, ReadableMetadataField, UnCached},
};
use core::marker::PhantomData;
use embedded_io::Read;

pub struct ReadablePayload<M: MetadataState, T: ReadableFrameField, R: Read> {
    metadata: M,
    _field_iterator_marker: PhantomData<T>,
    reader: R,
}

//--- Cache impls ---//
impl<M: ReadableMetadataField, T: ReadableFrameField, R: Read> ReadablePayload<Cached<M>, T, R> {
    pub fn from_metadata(reader: R, metadata: M) -> Self {
        Self {
            metadata: Cached { metadata },
            _field_iterator_marker: PhantomData,
            reader,
        }
    }
    pub fn metadata(&self) -> &M {
        &self.metadata.metadata
    }
}

impl<M: ReadableMetadataField, T: ReadableFrameField, R: Read> IntoIterator
    for ReadablePayload<Cached<M>, T, R>
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
impl<T: ReadableFrameField, R: Read> ReadablePayload<UnCached, T, R> {
    pub fn new(reader: R) -> Self {
        Self {
            metadata: UnCached,
            _field_iterator_marker: PhantomData,
            reader,
        }
    }
    pub fn load<M: ReadableMetadataField>(
        mut self,
    ) -> Result<ReadablePayload<Cached<M>, T, R>, ReadError<R::Error>>
    where
        [(); M::SIZE]:,
    {
        let mut buf = [0u8; M::SIZE];
        self.reader.read_exact(&mut buf)?;
        let metadata = M::read(buf).map_err(|_| ReadError::InvalidCast)?;
        let cache = Cached { metadata };
        let payload = ReadablePayload {
            metadata: cache,
            _field_iterator_marker: PhantomData,
            reader: self.reader,
        };

        Ok(payload)
    }
}
