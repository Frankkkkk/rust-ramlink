//! RAM-based, producer-consumer, one-way communication using a ring buffer
//!
//! This crate provides a way to transmit information from a producer (for example
//! a microcontroller) to a consumer (a debbuging host) using a shared RAM memory.
//! Usually, it is possible to read the RAM of microcontrollers using debugging
//! interfaces (JTAG, UPDI, ..).
//!
//! This way, it's possible to transmit information through the debugging interface
//! without relying on an additional UART.

#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

#[allow(dead_code)]
const RB_MAGIC: [u8; 3] = [0x88, 0x88, 0x88]; // XXX to share amongst prod/cons

#[cfg(feature = "consumer")]
pub mod consumer;

#[cfg(feature = "producer")]
pub mod producer;
