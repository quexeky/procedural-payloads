use embedded_io_async::{Read, Write};
use zerocopy::{Immutable, IntoBytes};

use crate::{
    read::{
        asynchronous::AsyncReadablePayload,
        error::ReadError,
        fields::ReadableFrameField,
        metadata::{Cached, ReadableMetadataField, UnCached},
    },
    write::{
        asynchronous::AsyncWritablePayload,
        error::WriteError,
        fields::WritableFrameField,
        metadata::{NotWritten, WritableMetadataField, Written},
    },
};

/// A duplex async transport for reading and writing procedural payloads.
///
/// The client does not own framing or transport policy: it delegates to the
/// payload readers and writers over the supplied Embassy-compatible streams.
pub struct AsyncClient<R, W> {
    reader: R,
    writer: W,
}

impl<R, W> AsyncClient<R, W> {
    /// Creates a client from independently owned receive and transmit streams.
    pub const fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    /// Borrows the receive and transmit streams independently.
    pub fn split(&mut self) -> (&mut R, &mut W) {
        (&mut self.reader, &mut self.writer)
    }

    /// Returns the streams after the client is no longer needed.
    pub fn into_parts(self) -> (R, W) {
        (self.reader, self.writer)
    }
}

impl<R: Read, W> AsyncClient<R, W> {
    /// Starts reading a payload and asynchronously loads its metadata prefix.
    pub async fn read_payload<M, T>(
        &mut self,
    ) -> Result<AsyncReadablePayload<Cached<M>, T, &mut R>, ReadError<R::Error>>
    where
        M: ReadableMetadataField,
        T: ReadableFrameField,
        [(); M::SIZE]:,
    {
        AsyncReadablePayload::<UnCached, T, _>::new(&mut self.reader)
            .load::<M>()
            .await
    }
}

impl<R, W: Write> AsyncClient<R, W> {
    /// Starts writing a payload by asynchronously emitting its metadata prefix.
    pub async fn write_payload<M, T>(
        &mut self,
        metadata: M,
    ) -> Result<AsyncWritablePayload<M, Written, T, &mut W>, WriteError<W::Error>>
    where
        M: WritableMetadataField + IntoBytes + Immutable,
        T: WritableFrameField + IntoBytes + Immutable,
    {
        AsyncWritablePayload::<M, NotWritten, T, _>::new(&mut self.writer)
            .begin(metadata)
            .await
    }
}
