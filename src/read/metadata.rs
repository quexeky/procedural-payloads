use core::{marker::PhantomData, mem::MaybeUninit, ops::Deref};
use embedded_io::Read;

use crate::read::{error::Error, readable::Readable};

pub trait ReadableMetadataField: Readable {
    fn num_fields(&self) -> usize;
}

pub trait MetadataState {}
pub struct Cached;
impl MetadataState for Cached {}
pub struct UnCached;
impl MetadataState for UnCached {}

pub struct MetadataCache<S: MetadataState, M: ReadableMetadataField> {
    _state: PhantomData<S>,
    metadata: MaybeUninit<M>,
}

// --- Cached impls --- //

impl<M: ReadableMetadataField> Deref for MetadataCache<Cached, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        unsafe { self.metadata.assume_init_ref() }
    }
}

impl<M: ReadableMetadataField> MetadataCache<Cached, M> {
    pub fn new_init(metadata: M) -> Self {
        Self {
            _state: PhantomData,
            metadata: MaybeUninit::new(metadata),
        }
    }
}

// --- UnCached impls --- //

impl<M: ReadableMetadataField> MetadataCache<UnCached, M> {
    pub const fn new() -> Self {
        Self {
            _state: PhantomData,
            metadata: MaybeUninit::uninit(),
        }
    }
    pub fn load<R: Read>(self, reader: &mut R) -> Result<MetadataCache<Cached, M>, Error<R::Error>>
    where
        [(); M::SIZE]:,
    {
        let mut buf = [0u8; M::SIZE];
        reader.read_exact(&mut buf)?;

        Ok(MetadataCache {
            _state: PhantomData,
            metadata: MaybeUninit::new(M::read::<R>(buf)?),
        })
    }
}

impl<M: ReadableMetadataField> Default for MetadataCache<UnCached, M> {
    fn default() -> Self {
        Self::new()
    }
}
