//! This module contains the consumer code needed to read data
//! from the [`producer`] ring buffer
//! <br>
//! In order to work, you must:
//! - Implement the `MemoryReader` trait
//! - Know the exact memory location of the RB struct defined in your producer
//! <br>
//! # Example
//! Implement the trait on your specific device
//! ```
//!  struct mk2<'a> {
//!      dev: JtagIceMkii<'a>,
//!  }
//!
//!  impl<'a> ramlink::consumer::MemoryReader for mk2<'a> {
//!      fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> Result<(), String> {
//!          for i in 0..buffer.len() {
//!              let byte = self.dev.read_ram_byte((address + i) as u16).unwrap();
//!              buffer[i] = byte;
//!          }
//!          Ok(())
//!      }
//!      fn write_memory(&mut self, address: usize, value: u8) -> Result<(), String> {
//!          self.dev.write_ram_byte(address as u16, value);
//!          Ok(())
//!      }
//!  }
//!```
//! Initialize it. In this example, the producer device is an AVR Attiny402, and the RB struct
//! is stored at address `0x3f0e`
//! ```
//!    let mm = mk2 { dev: dgr };
//!
//!    let mut rb = ramlink::consumer::ProducerDevice::new(Box::new(mm), 0x3f0e).unwrap();
//! ```
//! and start reading:
//! ```
//!    while true {
//!        let r = rb.read_bytes();
//!        if r.len() > 0 {
//!            println!("I READ {:02x?}", r);
//!        }
//!   }
//! ```

#![warn(missing_docs)]

use alloc::{boxed::Box, vec::Vec};
use core::fmt::Error;
use std::println;
extern crate alloc;
extern crate std;

use super::RB_MAGIC;

/// Error for consumer
#[derive(Debug)]
pub struct ConsumerError(ConsumerErrorKind);

/// Error types that the consumer can have
#[derive(Debug)]
pub enum ConsumerErrorKind {
    /// The magic marker was not found at the start of the struct. Maybe the RAM address is wrong
    MagicMarkerNotFound,
    /// The ring buffer reports a size 0. This should not happen
    RingBufferSizeNull,
    /// There was an error reading the memory address
    ReadMemoryError(Error),
    /// There was an error writing to the memory address
    WriteMemoryError(Error),
}

/// Trait that the consumer interface (JTAG, UPDI, ...) must support
pub trait MemoryReader {
    /// Reads [`buffer`] elements starting from `address`.
    fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> Result<(), Error>;
    /// Writes `value` at the specified memory address
    fn write_memory(&mut self, address: usize, value: u8) -> Result<(), Error>;
}

/// Represents a producer device, consisting of a reader, a ring buffer RAM address, and a ring buffer size
pub struct ProducerDevice<'a> {
    /// The location in RAM of the [`RB`] struct
    ram_start: usize,
    /// The memory reader implementation
    memory_reader: Box<dyn 'a + MemoryReader>,
    /// The size of the ring buffer. Will be checked against size defined in the [`RB`] struct.
    rb_size: u8,
}

const ADDR_SIZE: usize = 3;
const ADDR_PROD: usize = 4;
const ADDR_CONS: usize = 5;
const ADDR_BUFF: usize = 6;

impl<'a> ProducerDevice<'a> {
    /// Initiates a new ProducerDevice. Connects to the [`RB`] struct and checks that the
    /// magic markers are present, etc.
    pub fn new(
        mut memory_reader: Box<dyn MemoryReader>,
        ram_start_address: usize,
    ) -> Result<ProducerDevice<'a>, ConsumerError> {
        let mut magic_markers = [0; 3];
        memory_reader
            .read_memory(ram_start_address, &mut magic_markers)
            .map_err(|e| ConsumerError(ConsumerErrorKind::ReadMemoryError(e)))?;

        if magic_markers != RB_MAGIC {
            return Err(ConsumerError(ConsumerErrorKind::MagicMarkerNotFound));
        }

        let mut buf = [0; 1];
        memory_reader
            .read_memory(ram_start_address + ADDR_SIZE, &mut buf)
            .map_err(|e| ConsumerError(ConsumerErrorKind::ReadMemoryError(e)))?;

        let rb_size = buf[0];
        if rb_size == 0 {
            return Err(ConsumerError(ConsumerErrorKind::RingBufferSizeNull));
        }

        // XXX logging
        println!("The RB is of size {}", rb_size);

        Ok(ProducerDevice {
            ram_start: ram_start_address,
            memory_reader,
            rb_size,
        })
    }

    /// Reads one byte at the specified memory address. A wragger against [`read_memory`].
    fn read_one_byte(&mut self, address: usize) -> Result<u8, ConsumerError> {
        let mut buf = [0u8; 1];
        self.memory_reader
            .read_memory(address, &mut buf)
            .map_err(|e| ConsumerError(ConsumerErrorKind::ReadMemoryError(e)))?;
        Ok(buf[0])
    }

    /// Reads the maximum number of bytes from the RB struct. By doing so it
    /// consumes the bytes from the producer struct an frees some space in the process.
    pub fn read_bytes(&mut self) -> Result<Vec<u8>, ConsumerError> {
        let mut bytes: Vec<u8> = Vec::new();

        let prod_a = self.ram_start + ADDR_PROD;
        let cons_a = self.ram_start + ADDR_CONS;
        let buff_a = self.ram_start + ADDR_BUFF;

        let prod_v = self.read_one_byte(prod_a)?;
        let mut cons_v = self.read_one_byte(cons_a)?;

        while prod_v != cons_v {
            let buff_v = self.read_one_byte(buff_a + cons_v as usize)?;
            cons_v = (cons_v + 1) % self.rb_size;
            bytes.push(buff_v);
            self.memory_reader
                .write_memory(cons_a, cons_v)
                .map_err(|e| ConsumerError(ConsumerErrorKind::WriteMemoryError(e)))?;
        }
        Ok(bytes)
    }
}
