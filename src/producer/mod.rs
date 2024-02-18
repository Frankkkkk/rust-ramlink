use const_assert::{Assert, IsTrue};
use core::fmt;

use super::RB_MAGIC;

pub struct RB<const SIZE: usize> {
    _magic_marker: [u8; 3], // This eats 3 bytes for "nothing" but is useful for debuging purposes
    size: u8,
    //waiting: u8,
    producer: u8,
    consumer: u8,
    content: [u8; SIZE],
}

impl<const SIZE: usize> RB<SIZE>
where
    Assert<{ SIZE <= 255 }>: IsTrue,
{
    pub const fn new() -> RB<SIZE> {
        RB {
            _magic_marker: RB_MAGIC,
            size: SIZE as u8,
            //waiting: 2,
            producer: 0,
            consumer: 0,
            content: [0x13; SIZE],
        }
    }

    pub fn send_bytes_blocking(&mut self, data: &[u8]) {
        for elem in data.iter() {
            //self.waiting = 1;
            loop {
                let prod = unsafe { core::ptr::read_volatile(&self.producer) };
                let cons = unsafe { core::ptr::read_volatile(&self.consumer) };
                if (prod + 1) % self.size != cons {
                    break;
                }
            }
            //self.waiting = 0;

            self.content[self.producer as usize] = *elem;

            let next_p = (self.producer + 1) % self.size;
            self.producer = next_p;
        }
    }
}

impl<const SIZE: usize> fmt::Write for RB<SIZE>
where
    Assert<{ SIZE <= 255 }>: IsTrue,
{
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.send_bytes_blocking(s.as_bytes());
        Ok(())
    }
}
