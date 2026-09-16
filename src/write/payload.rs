use core::marker::PhantomData;

use embedded_io::Write;

use crate::write::{
    error::WriteError,
    fields::WritableFrameField,
    metadata::{MetadataWriteState, NotWritten, WritableMetadataField, Written},
};

pub struct WritablePayload<
    M: WritableMetadataField,
    S: MetadataWriteState,
    T: WritableFrameField,
    W: Write,
> {
    _metadata_type: PhantomData<M>,
    metadata_state: S,
    _frame_type: PhantomData<T>,
    writer: W,
}

impl<M: WritableMetadataField, T: WritableFrameField, W: Write>
    WritablePayload<M, NotWritten, T, W>
{
    pub fn new(writer: W) -> Self {
        Self {
            _metadata_type: PhantomData,
            metadata_state: NotWritten,
            _frame_type: PhantomData,
            writer,
        }
    }
    pub fn begin(
        mut self,
        metadata: M,
    ) -> Result<WritablePayload<M, Written, T, W>, WriteError<W::Error>> {
        let planned = metadata
            .num_fields()
            .checked_mul(T::SIZE)
            .and_then(|fields| fields.checked_add(size_of::<M>()))
            .ok_or(WriteError::TooMuchPlannedData)?;
        
        if planned >= 65536 {
            return Err(WriteError::TooMuchPlannedData);
        }
        metadata.write_to(&mut self.writer)?;

        Ok(WritablePayload {
            _metadata_type: PhantomData,
            metadata_state: Written::new(metadata.num_fields()),
            _frame_type: PhantomData,
            writer: self.writer,
        })
    }
}

impl<M: WritableMetadataField, T: WritableFrameField, W: Write>
    WritablePayload<M, Written, T, W>
{
    pub fn write_field(&mut self, field: T) -> Result<(), WriteError<W::Error>> {
        if self.metadata_state.fields_remaining == 0 {
            return Err(WriteError::ExcessData);
        }
        field.write_to(&mut self.writer)?;
        self.metadata_state.fields_remaining -= 1;
        Ok(())
    }
    pub fn fields_remaining(&self) -> usize {
        self.metadata_state.fields_remaining
    }
    pub fn finish(mut self) -> Result<W, WriteError<W::Error>> {
        if self.metadata_state.fields_remaining != 0 {
            return Err(WriteError::InsufficientDataWritten);
        }
        self.writer.flush()?;
        Ok(self.writer)
    }
}
