use std::{print, println};

use alloc::{boxed::Box, vec};
extern crate std;

extern crate alloc;

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

impl<'a> ProducerDevice<'a> {
    pub fn new(
        mut memory_reader: Box<dyn MemoryReader>,
        ram_start_address: usize,
    ) -> Result<ProducerDevice<'a>, ()> {
        //First of all, ensure that we have the correct base address of the RB
        let mut magic_markers = [0; 3];
        memory_reader.read_memory(ram_start_address, &mut magic_markers);

        const WANT_MAGIC: [u8; 3] = [0x88, 0x88, 0x88];
        if magic_markers != [0x88, 0x88, 0x88] {
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

    pub fn read_byte(&mut self) -> Option<u8> {
        let size = 3;
        let mut buf = vec![0; size];
        let b = self.memory_reader.read_memory(2, &mut buf);

        Some(8)
    }
}
