use core::marker::PhantomData;

use embedded_io_async::Read;

use crate::read::{
    error::ReadError,
    fields::ReadableFrameField,
    metadata::{Cached, MetadataState, ReadableMetadataField, UnCached},
};

/// A payload reader that loads metadata and asynchronously streams fixed-size fields.
pub struct AsyncReadablePayload<M: MetadataState, T: ReadableFrameField, R: Read> {
    metadata: M,
    reader: R,
    _field_type: PhantomData<T>,
}

impl<M: ReadableMetadataField, T: ReadableFrameField, R: Read>
    AsyncReadablePayload<Cached<M>, T, R>
{
    /// Starts reading fields after metadata has already been decoded.
    pub fn from_metadata(reader: R, metadata: M) -> Self {
        Self {
            metadata: Cached { metadata },
            reader,
            _field_type: PhantomData,
        }
    }

    /// Returns the metadata that describes this payload's fields.
    pub fn metadata(&self) -> &M {
        &self.metadata.metadata
    }

    /// Transfers the stream into an async field reader.
    pub fn into_fields(self) -> AsyncFieldReader<T, R>
    where
        [(); T::SIZE]:,
    {
        AsyncFieldReader::new(self.metadata.num_fields(), self.reader)
    }
}

impl<T: ReadableFrameField, R: Read> AsyncReadablePayload<UnCached, T, R> {
    /// Starts reading a payload whose metadata is at the beginning of the stream.
    pub fn new(reader: R) -> Self {
        Self {
            metadata: UnCached,
            reader,
            _field_type: PhantomData,
        }
    }

    /// Reads and validates the fixed-size metadata prefix.
    pub async fn load<M: ReadableMetadataField>(
        mut self,
    ) -> Result<AsyncReadablePayload<Cached<M>, T, R>, ReadError<R::Error>>
    where
        [(); M::SIZE]:,
    {
        let mut bytes = [0; M::SIZE];
        self.reader.read_exact(&mut bytes).await?;
        let metadata = M::read(bytes).map_err(|_| ReadError::InvalidCast)?;

        Ok(AsyncReadablePayload {
            metadata: Cached { metadata },
            reader: self.reader,
            _field_type: PhantomData,
        })
    }
}

/// Reads one fixed-size field at a time without blocking the executor.
pub struct AsyncFieldReader<T: ReadableFrameField, R: Read>
where
    [(); T::SIZE]:,
{
    elements_remaining: usize,
    errored: bool,
    reader: R,
    _field_type: PhantomData<T>,
}

impl<T: ReadableFrameField, R: Read> AsyncFieldReader<T, R>
where
    [(); T::SIZE]:,
{
    /// Creates a reader for exactly `field_count` consecutive fields.
    pub fn new(field_count: usize, reader: R) -> Self {
        Self {
            elements_remaining: field_count,
            errored: false,
            reader,
            _field_type: PhantomData,
        }
    }

    /// Reads the next field, returning `None` when all declared fields are consumed.
    ///
    /// Cancelling this future can leave a partially consumed field in the stream.
    pub async fn next_field(&mut self) -> Option<Result<T, ReadError<R::Error>>> {
        if self.errored || self.elements_remaining == 0 {
            return None;
        }

        let mut bytes = [0; T::SIZE];
        if let Err(error) = self.reader.read_exact(&mut bytes).await {
            self.errored = true;
            return Some(Err(ReadError::ReadExact(error)));
        }

        self.elements_remaining -= 1;
        match T::read(bytes) {
            Ok(field) => Some(Ok(field)),
            Err(_) => {
                self.errored = true;
                Some(Err(ReadError::InvalidCast))
            }
        }
    }

    /// Returns the underlying reader only after every declared field was read.
    pub fn finish(self) -> Result<R, ReadError<R::Error>> {
        if self.errored || self.elements_remaining != 0 {
            return Err(ReadError::InsufficientData);
        }
        Ok(self.reader)
    }
}

/// Reads a CRC-32-protected stream while hashing the bytes consumed from it.
pub struct AsyncCrc32Reader<R: Read> {
    reader: R,
    hasher: crc32fast::Hasher,
}

impl<R: Read> embedded_io::ErrorType for AsyncCrc32Reader<R> {
    type Error = R::Error;
}

impl<R: Read> Read for AsyncCrc32Reader<R> {
    async fn read(&mut self, bytes: &mut [u8]) -> Result<usize, Self::Error> {
        let count = self.reader.read(bytes).await?;
        self.hasher.update(&bytes[..count]);
        Ok(count)
    }
}

impl<R: Read> AsyncCrc32Reader<R> {
    /// Wraps a reader and starts a fresh CRC-32 calculation.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            hasher: crc32fast::Hasher::new(),
        }
    }

    /// Returns the CRC-32 of bytes read so far without consuming the reader.
    pub fn checksum(&self) -> u32 {
        self.hasher.clone().finalize()
    }

    /// Consumes and verifies the four-byte big-endian CRC trailer.
    pub async fn validate(mut self) -> Result<(), ReadError<R::Error>> {
        let mut checksum_bytes = [0; 4];
        self.reader.read_exact(&mut checksum_bytes).await?;
        let calculated = self.hasher.finalize();
        let read = u32::from_be_bytes(checksum_bytes);
        if calculated == read {
            Ok(())
        } else {
            Err(ReadError::InvalidCrc { calculated, read })
        }
    }

    /// Returns the inner reader and current checksum state.
    pub fn into_inner(self) -> (R, crc32fast::Hasher) {
        (self.reader, self.hasher)
    }
}
