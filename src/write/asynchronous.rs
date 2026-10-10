use core::marker::PhantomData;

use embedded_io_async::Write;
use zerocopy::{Immutable, IntoBytes};

use crate::write::{
    error::WriteError,
    fields::WritableFrameField,
    metadata::{MetadataWriteState, NotWritten, WritableMetadataField, Written},
};

/// Writes a metadata-prefixed stream of fixed-size fields asynchronously.
pub struct AsyncWritablePayload<
    M: WritableMetadataField,
    S: MetadataWriteState,
    T: WritableFrameField,
    W: Write,
> {
    metadata_state: S,
    writer: W,
    _metadata_type: PhantomData<M>,
    _field_type: PhantomData<T>,
}

impl<M, T, W> AsyncWritablePayload<M, NotWritten, T, W>
where
    M: WritableMetadataField + IntoBytes + Immutable,
    T: WritableFrameField + IntoBytes + Immutable,
    W: Write,
{
    /// Creates a writer that has not yet emitted its metadata prefix.
    pub fn new(writer: W) -> Self {
        Self {
            metadata_state: NotWritten,
            writer,
            _metadata_type: PhantomData,
            _field_type: PhantomData,
        }
    }

    /// Writes metadata and returns a writer limited to its declared field count.
    pub async fn begin(
        mut self,
        metadata: M,
    ) -> Result<AsyncWritablePayload<M, Written, T, W>, WriteError<W::Error>> {
        let planned_bytes = metadata
            .num_fields()
            .checked_mul(T::SIZE)
            .and_then(|fields| fields.checked_add(size_of::<M>()))
            .ok_or(WriteError::TooMuchPlannedData)?;

        if planned_bytes >= 65_536 {
            return Err(WriteError::TooMuchPlannedData);
        }

        self.writer.write_all(metadata.as_bytes()).await?;
        Ok(AsyncWritablePayload {
            metadata_state: Written::new(metadata.num_fields()),
            writer: self.writer,
            _metadata_type: PhantomData,
            _field_type: PhantomData,
        })
    }
}

impl<M, T, W> AsyncWritablePayload<M, Written, T, W>
where
    M: WritableMetadataField,
    T: WritableFrameField + IntoBytes + Immutable,
    W: Write,
{
    /// Writes one field, failing if metadata declared no fields remain.
    ///
    /// Cancelling this future can leave a partially written field in the stream.
    pub async fn write_field(&mut self, field: T) -> Result<(), WriteError<W::Error>> {
        if self.metadata_state.fields_remaining == 0 {
            return Err(WriteError::ExcessData);
        }
        self.writer.write_all(field.as_bytes()).await?;
        self.metadata_state.fields_remaining -= 1;
        Ok(())
    }

    /// Returns the number of declared fields that have not yet been written.
    pub fn fields_remaining(&self) -> usize {
        self.metadata_state.fields_remaining
    }

    /// Flushes and returns the writer once every declared field was written.
    pub async fn finish(mut self) -> Result<W, WriteError<W::Error>> {
        if self.metadata_state.fields_remaining != 0 {
            return Err(WriteError::InsufficientDataWritten);
        }
        self.writer.flush().await?;
        Ok(self.writer)
    }
}

/// Writes a CRC-32-protected stream and hashes only bytes accepted by the writer.
pub struct AsyncCrc32Writer<W: Write> {
    writer: W,
    hasher: crc32fast::Hasher,
}

impl<W: Write> embedded_io::ErrorType for AsyncCrc32Writer<W> {
    type Error = W::Error;
}

impl<W: Write> Write for AsyncCrc32Writer<W> {
    async fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        let written = self.writer.write(bytes).await?;
        self.hasher.update(&bytes[..written]);
        Ok(written)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.writer.flush().await
    }
}

impl<W: Write> AsyncCrc32Writer<W> {
    /// Wraps a writer and starts a fresh CRC-32 calculation.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            hasher: crc32fast::Hasher::new(),
        }
    }

    /// Returns the CRC-32 of bytes accepted so far without consuming the writer.
    pub fn checksum(&self) -> u32 {
        self.hasher.clone().finalize()
    }

    /// Writes a big-endian CRC-32 trailer, flushes, and returns the inner writer.
    pub async fn finish(mut self) -> Result<W, W::Error> {
        let checksum = self.hasher.finalize();
        self.writer.write_all(&checksum.to_be_bytes()).await?;
        self.writer.flush().await?;
        Ok(self.writer)
    }
}
