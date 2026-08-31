use core::marker::PhantomData;

use embedded_io::Write;

use crate::write::{
    error::Error,
    fields::WritableFrameField,
    metadata::{MetadataWriteState, NotWritten, WritableMetadataField, Written},
};

pub struct WritablePayload<
    const FIELD_SIZE: usize,
    M: WritableMetadataField,
    S: MetadataWriteState,
    T: WritableFrameField<FIELD_SIZE>,
    W: Write,
> {
    _metadata_type: PhantomData<M>,
    metadata_state: S,
    _frame_type: PhantomData<T>,
    writer: W,
}

impl<const FIELD_SIZE: usize, M: WritableMetadataField, T: WritableFrameField<FIELD_SIZE>, W: Write>
    WritablePayload<FIELD_SIZE, M, NotWritten, T, W>
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
    ) -> Result<WritablePayload<FIELD_SIZE, M, Written, T, W>, Error<W::Error>> {
        let num_fields = metadata.num_fields();
        if num_fields * FIELD_SIZE >= 65536 {
            return Err(Error::TooMuchPlannedData);
        }
        metadata.write_to(&mut self.writer)?;

        Ok(WritablePayload {
            _metadata_type: PhantomData,
            metadata_state: Written::new(num_fields),
            _frame_type: PhantomData,
            writer: self.writer,
        })
    }
}
impl<const FIELD_SIZE: usize, M: WritableMetadataField, T: WritableFrameField<FIELD_SIZE>, W: Write>
    WritablePayload<FIELD_SIZE, M, Written, T, W>
{
    pub fn write_field(&mut self, field: T) -> Result<(), Error<W::Error>> {
        let planned = self
            .metadata_state
            .fields_remaining
            .checked_mul(FIELD_SIZE)
            .ok_or(Error::TooMuchPlannedData)?;

        if planned >= 65536 {
            return Err(Error::TooMuchPlannedData);
        }
        self.metadata_state.fields_remaining -= 1;
        field.write_to(&mut self.writer)?;
        Ok(())
    }
    pub fn fields_remaining(&self) -> usize {
        self.metadata_state.fields_remaining
    }
    pub fn finish(mut self) -> Result<(), Error<W::Error>> {
        if self.metadata_state.fields_remaining != 0 {
            return Err(Error::InsufficientDataWritten);
        }
        self.writer.flush()?;
        Ok(())
    }
}
