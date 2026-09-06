use embedded_io::Read;
use zerocopy::TryFromBytes;

use crate::read::error::Error;

pub trait Readable: Sized {
    const SIZE: usize;
    fn read<R: Read>(data: [u8; Self::SIZE]) -> Result<Self, Error<R::Error>>;
}

impl<T: TryFromBytes> Readable for T {
    const SIZE: usize = size_of::<Self>();

    fn read<R: Read>(data: [u8; Self::SIZE]) -> Result<Self, Error<R::Error>> {
        Ok(T::try_read_from_bytes(&data)?)
    }
}
