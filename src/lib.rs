#![no_std]
#![no_main]

use cortex_m_semihosting::debug;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

pub mod commands;
pub mod device;
pub mod gpios;

//TODO: probably makes sense to move this to main and keep any logic for signalling
//      in the main binary rather than in the library crate
pub static SET_INT_OUT: Signal<CriticalSectionRawMutex, bool> = Signal::new();

pub mod prelude {
    pub use crate::commands;
    pub use crate::device;
    pub use crate::gpios;
    pub use crate::SET_INT_OUT;
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
