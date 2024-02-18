use alloc::{boxed::Box, vec::Vec};
use core::fmt::Error;
use std::println;
extern crate alloc;
extern crate std;

use super::RB_MAGIC;

#[derive(Debug)]
pub struct ConsumerError(ConsumerErrorKind);

#[derive(Debug)]
pub enum ConsumerErrorKind {
    MagicMarkerNotFound,
    RingBufferSizeNull,
    ReadMemoryError(Error),
    WriteMemoryError(Error),
}

pub trait MemoryReader {
    fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> Result<(), Error>;
    fn write_memory(&mut self, address: usize, value: u8) -> Result<(), Error>;
}

pub struct ProducerDevice<'a> {
    ram_start: usize,
    memory_reader: Box<dyn 'a + MemoryReader>,
    rb_size: u8,
}

const ADDR_SIZE: usize = 3;
const ADDR_PROD: usize = 4;
const ADDR_CONS: usize = 5;
const ADDR_BUFF: usize = 6;

impl<'a> ProducerDevice<'a> {
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

    fn read_one_byte(&mut self, address: usize) -> Result<u8, ConsumerError> {
        let mut buf = [0u8; 1];
        self.memory_reader
            .read_memory(address, &mut buf)
            .map_err(|e| ConsumerError(ConsumerErrorKind::ReadMemoryError(e)))?;
        Ok(buf[0])
    }

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
