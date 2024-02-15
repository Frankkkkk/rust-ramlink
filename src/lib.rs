#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
use const_assert::{Assert, IsTrue};
use core::fmt;

struct RB<const SIZE: usize> {
    // XXX SIZE should be ≤ 255. How to check that at compile time ?
    size: u8,
    waiting: u8,
    producer: u8,
    consumer: u8,
    //    content: [u8; SIZE],
    content: [u8; SIZE],
}

impl<const SIZE: usize> RB<SIZE>
//where
//    Assert<{ SIZE <= 255 }>: IsTrue,
{
    //const fn new() -> RB<SIZE> {
    const fn new() -> RB<SIZE> {
        /*
                RB {
                    size: 0x10,     //SIZE as u8,
                    producer: 0x11, //0, // ptr to
                    consumer: 0x12, //0,
                    content: [0x13; SIZE],
                }
        */
        RB {
            size: SIZE as u8, //0x10,     //SIZE as u8,
            waiting: 2,
            producer: 0, //0x11,   //0, // ptr to
            consumer: 0,
            //content: [0x13; 5],
            content: [0x13; SIZE],
        }
    }

    pub fn send_bytes_blocking(&mut self, data: &[u8]) {
        for elem in data.iter() {
            self.waiting = 1;
            loop {
                let prod = unsafe { core::ptr::read_volatile(&self.producer) };
                let cons = unsafe { core::ptr::read_volatile(&self.consumer) };
                if (prod + 1) % self.size != cons {
                    break;
                }
            }
            self.waiting = 0;

            self.content[self.producer as usize] = *elem;

            let next_p = (self.producer + 1) % self.size;
            self.producer = next_p;
        }
    }
}

impl<const SIZE: usize> fmt::Write for RB<SIZE>
//where
//    Assert<{ SIZE <= 255 }>: IsTrue,
{
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.send_bytes_blocking(s.as_bytes());
        /*
        for b in s.as_bytes().iter() {
            self.write_byte(*b);
        }
        */
        Ok(())
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
