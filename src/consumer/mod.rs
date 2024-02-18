use std::{println};

use alloc::{
    boxed::Box,
    vec::{Vec},
};
extern crate alloc;
extern crate std;

//use super::RB_MAGIC;

pub trait MemoryReader {
    fn read_memory(
        &mut self,
        address: usize,
        buffer: &mut [u8],
    ) -> Result<(), alloc::string::String>;
    fn write_memory(&mut self, address: usize, value: u8) -> Result<(), alloc::string::String>;
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
    ) -> Result<ProducerDevice<'a>, ()> {
        //First of all, ensure that we have the correct base address of the RB
        let mut magic_markers = [0; 3];
        memory_reader.read_memory(ram_start_address, &mut magic_markers);

        const WANT_MAGIC: [u8; 3] = [0x88, 0x88, 0x88]; // XXX to share amongst prod/cons
        if magic_markers != WANT_MAGIC {
            //return Err("Magic markers don't start with 0x88".to_string());
            panic!(
                "Magic markers arent {:02x?}: {:02x?}",
                WANT_MAGIC, magic_markers
            );
            return Err(());
        }

        //Get the size of the RB
        let mut buf = [0; 1];
        memory_reader.read_memory(ram_start_address + 3, &mut buf);
        let rb_size = buf[0];
        if rb_size == 0 {
            return Err(());
        }

        println!("The RB is of size {}", rb_size);

        Ok(ProducerDevice {
            ram_start: ram_start_address,
            memory_reader,
            rb_size,
        })
    }

    fn read_one_byte(&mut self, address: usize) -> Result<u8, ()> {
        // Basically the same as read_memory, but only reads
        // one byte. Easier than allocating a buffer, etc..

        let mut buf = [0u8; 1];
        match self.memory_reader.read_memory(address, &mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(_x) => Err(()),
        }
    }

    pub fn read_bytes(&mut self) -> Vec<u8> {
        let mut bytes: Vec<u8> = alloc::vec![];

        //for mem_addr in 0x3f00..0x3f0f {
        let prod_a = self.ram_start + ADDR_PROD;
        let cons_a = self.ram_start + ADDR_CONS;
        let buff_a = self.ram_start + ADDR_BUFF;

        let prod_v = self.read_one_byte(prod_a).unwrap();
        let mut cons_v = self.read_one_byte(cons_a).unwrap();

        while prod_v != cons_v {
            let buff_a = buff_a + cons_v as usize;
            let buff_v = self.read_one_byte(buff_a).unwrap(); // XXX TODO: we could read the whole range
            cons_v = (cons_v + 1) % 5;
            bytes.push(buff_v);
            //            println!("RCVD: {:02x?}", buff_v);
            self.memory_reader.write_memory(cons_a, cons_v);
        }
        bytes
    }
}
