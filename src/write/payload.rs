use core::marker::PhantomData;

use embedded_io::Write;

use crate::write::{
    error::Error,
    fields::WritableFrameField,
    metadata::{MetadataWriteState, NotWritten, WritableMetadataField, Written},
};

pub struct WritablePayload<
    'a,
    M: WritableMetadataField,
    S: MetadataWriteState,
    T: WritableFrameField,
    W: Write + ?Sized,
> {
    _metadata_type: PhantomData<M>,
    metadata_state: S,
    _frame_type: PhantomData<T>,
    writer: &'a mut W,
}

impl<
    'a,
    M: WritableMetadataField,
    T: WritableFrameField,
    W: Write + ?Sized,
> WritablePayload<'a, M, NotWritten, T, W>
{
    pub fn new(writer: &'a mut W) -> Self {
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
    ) -> Result<WritablePayload<'a, M, Written, T, W>, Error<W::Error>> {
        let num_fields = metadata.num_fields();
        let planned = num_fields
            .checked_mul(T::SIZE)
            .ok_or(Error::TooMuchPlannedData)?;

        if planned >= 65536 {
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

impl<
    'a,
    M: WritableMetadataField,
    T: WritableFrameField,
    W: Write + ?Sized,
> WritablePayload<'a, M, Written, T, W>
{
    pub fn write_field(&mut self, field: T) -> Result<(), Error<W::Error>> {
        if self.metadata_state.fields_remaining == 0 {
            return Err(Error::ExcessData);
        }
        self.metadata_state.fields_remaining -= 1;
        field.write_to(&mut self.writer)?;
        Ok(())
    }
    pub fn fields_remaining(&self) -> usize {
        self.metadata_state.fields_remaining
    }
    pub fn finish(self) -> Result<&'a mut W, Error<W::Error>> {
        if self.metadata_state.fields_remaining != 0 {
            return Err(Error::InsufficientDataWritten);
        }
        self.writer.flush()?;
        Ok(self.writer)
    }
}
