use core::marker::PhantomData;

use embedded_io::Write;

use crate::write::{error::Error, fields::WritableFrameField, metadata::WritableMetadataField};

pub trait MetadataWriteState {}
pub struct Written {
    fields_to_write: usize,
}
impl MetadataWriteState for Written {}
pub struct NotWritten;
impl MetadataWriteState for NotWritten {}

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
            metadata_state: Written {
                fields_to_write: num_fields,
            },
            _frame_type: PhantomData,
            writer: self.writer,
        })
    }
}
impl<const FIELD_SIZE: usize, M: WritableMetadataField, T: WritableFrameField<FIELD_SIZE>, W: Write>
    WritablePayload<FIELD_SIZE, M, Written, T, W>
{
    pub fn write_field(&mut self, field: T) -> Result<(), Error<W::Error>> {
        if self.metadata_state.fields_to_write == 0 {
            return Err(Error::ExcessData);
        }
        self.metadata_state.fields_to_write -= 1;
        field.write_to(&mut self.writer)?;
        Ok(())
    }
    pub fn finish(mut self) -> Result<(), Error<W::Error>> {
        if self.metadata_state.fields_to_write != 0 {
            return Err(Error::InsufficientDataWritten);
        }
        self.writer.flush()?;
        Ok(())
    }
}
