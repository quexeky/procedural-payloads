#![no_std]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![doc = "Allocation-free payload serialization for embedded streams."]

#[cfg(feature = "async")]
pub mod client;
pub mod read;
pub mod write;
