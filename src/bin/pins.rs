#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::Flex;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut pin = Flex::new(p.PIN_9);
    pin.set_as_input();

    info!("Running");
    let mut level = pin.is_high();
    loop {
        Timer::after_millis(100).await;
        let new_level = pin.is_high();
        if new_level != level {
            level = new_level;
            info!("Level: {}", level);
        }
    }
}
