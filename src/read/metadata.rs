use core::{marker::PhantomData, mem::MaybeUninit, ops::Deref};
use embedded_io::{Read, ReadExactError};

pub trait ReadableMetadataField<const SIZE: usize>: From<[u8; SIZE]> {
    fn num_fields(&self) -> usize;
}

pub trait MetadataState {}
pub struct Cached;
impl MetadataState for Cached {}
pub struct UnCached;
impl MetadataState for UnCached {}

pub struct MetadataCache<const SIZE: usize, S: MetadataState, M: ReadableMetadataField<SIZE>> {
    _state: PhantomData<S>,
    metadata: MaybeUninit<M>,
}

// --- Cached impls --- //

impl<const SIZE: usize, M: ReadableMetadataField<SIZE>> Deref for MetadataCache<SIZE, Cached, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        unsafe { self.metadata.assume_init_ref() }
    }
}

impl<const SIZE: usize, M: ReadableMetadataField<SIZE>> MetadataCache<SIZE, Cached, M> {
    pub fn new_init(metadata: M) -> Self {
        Self {
            _state: PhantomData,
            metadata: MaybeUninit::new(metadata),
        }
    }
}

// --- UnCached impls --- //

impl<const SIZE: usize, M: ReadableMetadataField<SIZE>> MetadataCache<SIZE, UnCached, M> {
    pub const fn new() -> Self {
        Self {
            _state: PhantomData,
            metadata: MaybeUninit::uninit(),
        }
    }
    pub fn load<R: Read>(
        self,
        reader: &mut R,
    ) -> Result<MetadataCache<SIZE, Cached, M>, ReadExactError<R::Error>> {
        let mut buf = [0; SIZE];
        reader.read_exact(&mut buf)?;
        Ok(MetadataCache {
            _state: PhantomData,
            metadata: MaybeUninit::new(M::from(buf)),
        })
    }
}

impl<const SIZE: usize, M: ReadableMetadataField<SIZE>> Default
    for MetadataCache<SIZE, UnCached, M>
{
    fn default() -> Self {
        Self::new()
    }
}
