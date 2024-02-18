#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

const RB_MAGIC: [u8; 3] = [0x88, 0x88, 0x88]; // XXX to share amongst prod/cons

#[cfg(feature = "consumer")]
pub mod consumer;

#[cfg(feature = "producer")]
pub mod producer;
