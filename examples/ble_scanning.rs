#![no_std]
#![feature(alloc)]
#[allow(unused)]
#[macro_use]
extern crate alloc;
extern crate corepack;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tock;

use tock::ble_parser;
use tock::led;
use tock::simple_ble::BleCallback;
use tock::simple_ble::BleDriver;
use tock::syscalls;

#[derive(Deserialize)]
struct LedCommand {
    pub nr: u8,
    pub st: bool,
}

// Prevents the compiler from dropping the subscription too early.
#[allow(unreachable_code)]
fn main() {
    let mut shared_buffer = BleDriver::create_scan_buffer();
    let mut _my_buffer = BleDriver::create_scan_buffer();
    let _shared_memory = BleDriver::share_memory(&mut shared_buffer).unwrap();

    let mut callback = BleCallback::new(|_: usize, _: usize| {
        _shared_memory.read_bytes(&mut _my_buffer);
        match ble_parser::find(&_my_buffer, tock::simple_ble::gap_data::SERVICE_DATA as u8)
            .and_then(|x| ble_parser::extract_for_service([91, 79], x))
        {
            Some(_payload) => {
                let msg: Result<LedCommand, _> = corepack::from_bytes(&_payload);
                match msg {
                    Ok(msg) => {
                        let led = led::get(msg.nr as isize);
                        match led {
                            Some(led) => {
                                led.set_state(msg.st);
                            }
                            None => (),
                        }
                    }
                    _ => (),
                }
            }
            None => (),
        }
    });

    let _subscription = BleDriver::start(&mut callback);

    loop {
        syscalls::yieldk();
    }

    _subscription.unwrap();
}
