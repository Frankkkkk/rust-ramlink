# rust-ramlink

<!-- cargo-rdme start -->

RAM-based, producer-consumer, one-way communication using a ring buffer

This no-std crate provides a way to transmit information from a producer (for example
a microcontroller) to a consumer (a debbuging host) using a shared RAM memory.
Usually, it is possible to read the RAM of microcontrollers using debugging
interfaces (JTAG, UPDI, ..).

This way, it's possible to transmit information through the debugging interface
without relying on an additional UART.
## Example
### Producer (AVR, PIC, microcontroller, ...)
Add this crate to your project, don't forget to enable the `producer` feature
`cargo add ramlink -F producer`

Then create an ring buffer of size 5:
In order to access it safely, we wrap it around a Mutex and a RefCell:
```rust
  use avr_device::interrupt::{self, Mutex};
  use core::cell::{Cell, RefCell};
  use ramlink::producer::RB;

  static RING_BUF: Mutex<RefCell<RB<5>>> = Mutex::new(RefCell::new(RB::<5>::new()));
```
you can then send data to your consumer:
```rust
  interrupt::free(|cs| {
    RING_BUF
    .borrow(cs)
    .borrow_mut()
    .send_bytes_blocking(&[temperature, current]);
  });
```
### Consumer (laptop with JTAG/UPDI/… interface)
Add this crate to you project, don't forget to enable the `consumer` feature
`cargo add ramlink -F consumer`
Implement the trait for your specific device
```rust
 struct mk2<'a> {
     dev: JtagIceMkii<'a>,
 }

 impl<'a> ramlink::consumer::MemoryReader for mk2<'a> {
     fn read_memory(&mut self, address: usize, buffer: &mut [u8]) -> Result<(), String> {
         for i in 0..buffer.len() {
             let byte = self.dev.read_ram_byte((address + i) as u16).unwrap();
             buffer[i] = byte;
         }
         Ok(())
     }
     fn write_memory(&mut self, address: usize, value: u8) -> Result<(), String> {
         self.dev.write_ram_byte(address as u16, value);
         Ok(())
     }
 }
```
Initialize it. In this example, the producer device is an AVR Attiny402, and the RB struct
is stored at address `0x3f0e`
```rust
   let mm = mk2 { dev: dgr };

   let mut rb = ramlink::consumer::ProducerDevice::new(Box::new(mm), 0x3f0e).unwrap();
```
and start reading:
```rust
   while true {
       let r = rb.read_bytes();
       if r.len() > 0 {
           println!("I READ {:02x?}", r);
       }
  }
```

<!-- cargo-rdme end -->

# Contributing

Given that I'm a rust noob, don't hesitate to raise issues or propose merge requests to make this code better and make me improve :-). Suggestions welcome !

