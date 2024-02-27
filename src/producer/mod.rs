//! This module contains the [`RB`] struct that is used to allocate
//! a space in memory that is used to transmit information from this producer
//! (a microcontroller most of the time) to a consumer (laptop JTAG connected
//! to the microcontroller).
//! <br>
//! It is necessary that the RB struct is precisely located in RAM so that you
//! know which address to query from the [`../consumer`] side.
//! # Examples
//! The following creates the [`RB`] struct of size **5** as a static variable. In order to
//! access it safely, we wrap it around a Mutex and a RefCell:
//! ```
//!   use avr_device::interrupt::{self, Mutex};
//!   use core::cell::{Cell, RefCell};
//!   use ramlink::producer::RB;
//!
//!   static RING_BUF: Mutex<RefCell<RB<5>>> = Mutex::new(RefCell::new(RB::<5>::new()));
//! ```
//! data can then be sent to it:
//! ```
//!   interrupt::free(|cs| {
//!     RING_BUF
//!     .borrow(cs)
//!     .borrow_mut()
//!     .send_bytes_blocking(&[temperature, current]);
//!   });
//!```

#![warn(missing_docs)]
use const_assert::{Assert, IsTrue};
use core::fmt;

use super::RB_MAGIC;

/// The RingBuffer struct that will contain our message to be sent.
/// Some fields are read only while others are written by the consumer (host, JTAG, ...)
pub struct RB<const SIZE: usize> {
    /// This eats 3 bytes for "nothing" but is useful for debuging purposes to ensure that the RAM address is correct
    _magic_marker: [u8; 3],
    /// Size of the ring buffer. Could be removed if both parties agree on a defined size
    size: u8,
    /// Producer slot
    producer: u8,
    /// Consumer slot. If producer = consumer, ring buffer is empty
    consumer: u8,
    /// The actual buffer
    content: [u8; SIZE],
}

impl<const SIZE: usize> RB<SIZE>
where
    Assert<{ SIZE <= 255 }>: IsTrue,
{
    /// Returns a new ring buffer of size `SIZE`
    pub const fn new() -> RB<SIZE> {
        RB {
            _magic_marker: RB_MAGIC,
            size: SIZE as u8,
            producer: 0,
            consumer: 0,
            content: [0x13; SIZE],
        }
    }

    /// Sends bytes on the ring buffer. This is blocking. If the
    /// ring buffer is full, it will wait for more space before moving on.
    /// This busy-waits for now.
    /// TODO: Add a non-blocking (lossy) method + interrupt based one ?
    pub fn send_bytes_blocking(&mut self, data: &[u8]) {
        for elem in data.iter() {
            loop {
                let prod = unsafe { core::ptr::read_volatile(&self.producer) };
                let cons = unsafe { core::ptr::read_volatile(&self.consumer) };
                if (prod + 1) % self.size != cons {
                    break;
                }
            }

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
    /// Implements write_src so we can use the write! macro
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.send_bytes_blocking(s.as_bytes());
        Ok(())
    }
}
