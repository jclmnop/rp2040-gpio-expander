#![no_std]
#![no_main]

use cortex_m_semihosting::debug;
use embassy_executor::{Executor, InterruptExecutor};
use embassy_rp::peripherals::{PIN_2, PIN_25, PIN_26};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use static_cell::StaticCell;

#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

pub mod commands;
pub mod device;
pub mod gpios;
pub mod tasks;

pub static SET_INT_OUT: Signal<CriticalSectionRawMutex, bool> = Signal::new();
#[allow(non_camel_case_types)]
pub type P_INT_OUT = PIN_26;
#[allow(non_camel_case_types)]
pub type P_LED = PIN_25;
#[allow(non_camel_case_types)]
pub type P_EN_OUT = PIN_2;

pub const ADDRESS: u8 = 0x20;
pub const DEFAULT_PIN_MODES: [u8; 2] = [0b0000_0001, 0b1111_0000];
pub static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();
pub static LED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub mod prelude {
    pub use crate::commands;
    pub use crate::device;
    pub use crate::gpios;
    pub use crate::tasks;
    pub use crate::{ADDRESS, DEFAULT_PIN_MODES, EXECUTOR, EXECUTOR_HIGH, LED};
    pub use crate::{P_EN_OUT, P_INT_OUT, P_LED, SET_INT_OUT};
    pub use defmt::*;
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes a semihosting-capable debug tool exit
/// with status code 0.
pub fn exit() -> ! {
    loop {
        debug::exit(debug::EXIT_SUCCESS);
    }
}

/// Hardfault handler.
///
/// Terminates the application and makes a semihosting-capable debug tool exit
/// with an error. This seems better than the default, which is to spin in a
/// loop.
#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}

// TODO: figure out how to apply link args to unit tests
// #[cfg(test)]
// #[defmt_test::tests]
// mod tests {
//     // use crate::device::tests;
//     use defmt::assert;
//
//     #[test]
//     fn it_works() {
//         assert!(true)
//     }
// }
