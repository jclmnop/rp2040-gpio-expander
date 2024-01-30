//! Pulsating LED example for reference

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::pwm::{Config, Pwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut c: Config = Default::default();
    c.top = 0x8000;
    c.compare_a = 8;
    let mut pwm = Pwm::new_output_a(p.PWM_CH3, p.PIN_22, c.clone());

    loop {
        info!("current LED duty cycle: {}/32768", c.compare_a);
        Timer::after_millis(10).await;
        c.compare_a = if c.compare_a < 0x8000 {
            c.compare_a + 100
        } else {
            c.invert_a = !c.invert_a;
            0
        };
        pwm.set_config(&c);
    }
}
