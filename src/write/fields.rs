use zerocopy::{Immutable, IntoBytes};

use crate::write::writeable::Writeable;

pub trait WritableFrameField: Writeable + Sized {
    const SIZE: usize;
}

impl<T: IntoBytes + Immutable> WritableFrameField for T {
    const SIZE: usize = size_of::<T>();
}