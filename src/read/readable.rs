use zerocopy::TryFromBytes;

/// Marker error: the bytes were not a valid representation of the target type.
/// Callers should map this to their own error type
#[derive(Debug)]
pub struct ConversionError;

pub trait Readable: Sized {
    const SIZE: usize;
    fn read(data: [u8; Self::SIZE]) -> Result<Self, ConversionError>;
}

impl<T: TryFromBytes> Readable for T {
    const SIZE: usize = size_of::<Self>();

    fn read(data: [u8; Self::SIZE]) -> Result<Self, ConversionError> {
        T::try_read_from_bytes(&data).map_err(|_| ConversionError)
    }
}
