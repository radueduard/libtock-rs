#![no_std]

use core::cell::Cell;
use libtock_platform::{
    share, subscribe::OneId, DefaultConfig, ErrorCode, Subscribe, Syscalls, Upcall,
};

pub struct Humidity<S: Syscalls>(S);

impl<S: Syscalls> Humidity<S> {
    /// Returns Ok() if the driver was present.This does not necessarily mean
    /// that the driver is working.
    pub fn exists() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, EXISTS, 0, 0).to_result()
    }

    /// Initiate a humidity measurement.
    ///
    /// This function is used both for synchronous and asynchronous readings

    pub fn humidity_read() -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, READ_HUMIDITY, 0, 0).to_result()
    }

    /// Register an events listener
    pub fn register_listener<'share, F: Fn(i32)>(
        listener: &'share HumidityListener<F>,
        subscribe: share::Handle<Subscribe<'share, S, DRIVER_NUM, 0>>,
    ) -> Result<(), ErrorCode> {
        S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, listener)
    }

    /// Unregister the events listener
    pub fn unregister_listener() {
        S::unsubscribe(DRIVER_NUM, 0)
    }

    /// Initiate a synchronous humidity measurement.
    /// ## Hello
    /// Returns Ok(humidity_value) if the operation was successful
    /// humidity_value is returned in hundreds of centigrades
    pub fn humidity_read_sync() -> Result<i32, ErrorCode> {
        let humidity_cell: Cell<Option<i32>> = Cell::new(None);
        let listener = HumidityListener(|hum_val| {
            humidity_cell.set(Some(hum_val));
        });
        share::scope(|subscribe| {
            if let Ok(()) = Self::register_listener(&listener, subscribe) {
                if let Ok(()) = Self::humidity_read() {
                    while humidity_cell.get() == None {
                        S::yield_wait();
                    }
                }
            }
        });

        match humidity_cell.get() {
            None => Err(ErrorCode::Busy),
            Some(hum_val) => Ok(hum_val),
        }
    }
}

pub struct HumidityListener<F: Fn(i32)>(pub F);
impl<F: Fn(i32)> Upcall<OneId<DRIVER_NUM, 0>> for HumidityListener<F> {
    fn upcall(&self, hum_val: u32, _arg1: u32, _arg2: u32) {
        self.0(hum_val as i32)
    }
}

#[cfg(test)]
mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x60001;

// Command IDs

const EXISTS: u32 = 0;
const READ_HUMIDITY: u32 = 1;
