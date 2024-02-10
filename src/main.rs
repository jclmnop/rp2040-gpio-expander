#![no_std]
#![no_main]

use defmt::*;
use device::Device;
use embassy_executor::Executor;

use embassy_rp::gpio::{Level, Output};

use embassy_rp::interrupt::{InterruptExt, Priority};
use embassy_rp::peripherals::I2C0;
use embassy_rp::{bind_interrupts, i2c, i2c_slave, interrupt};

use gpios::{PinGroup0, PinGroup1};
use rp_2040_gpio_expander::prelude::*;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = embassy_rp::init(Default::default());

    interrupt::SWI_IRQ_0.set_priority(Priority::P2);
    let high_spawner = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_0);

    let en_out: P_EN_OUT = peripherals.PIN_2;
    unwrap!(high_spawner.spawn(tasks::trigger_en_out(en_out)));

    let int_out: P_INT_OUT = peripherals.PIN_26;
    unwrap!(high_spawner.spawn(tasks::trigger_int_out(int_out)));

    let executor = EXECUTOR.init(Executor::new());
    let led = Output::new(peripherals.PIN_25, Level::Low);

    let sda = peripherals.PIN_4;
    let scl = peripherals.PIN_5;

    let mut config = i2c_slave::Config::default();
    config.addr = ADDRESS as u16;
    let slave = i2c_slave::I2cSlave::new(peripherals.I2C0, scl, sda, Irqs, config);
    let gpio_group_0 = PinGroup0::new(
        peripherals.PIN_6,
        peripherals.PIN_7,
        peripherals.PIN_8,
        peripherals.PIN_9,
        peripherals.PIN_10,
        peripherals.PIN_11,
        peripherals.PIN_12,
        peripherals.PIN_13,
    );
    let gpio_group_1 = PinGroup1::new(
        peripherals.PIN_14,
        peripherals.PIN_15,
        peripherals.PIN_16,
        peripherals.PIN_17,
        peripherals.PIN_18,
        peripherals.PIN_19,
        peripherals.PIN_20,
        peripherals.PIN_21,
    );
    let device = Device::new(gpio_group_0, gpio_group_1);

    executor.run(|spawner| {
        unwrap!(spawner.spawn(tasks::led_task(led)));
        unwrap!(spawner.spawn(tasks::i2c_task(slave, device)));
    })
}
