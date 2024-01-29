#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{I2C0, I2C1, PIN_22};
use embassy_rp::{bind_interrupts, i2c, i2c_slave};
use embassy_rp::gpio::{Pin, Level, Output};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Timer;
use embedded_hal_async::i2c::I2c;
use embassy_futures::select::{Either, select};
use {defmt_rtt as _, panic_probe as _};

const ADDRESS: u8 = 0x20;
static LED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let led = Output::new(peripherals.PIN_22, Level::Low);

    unwrap!(spawner.spawn(led_task(led)));

    let sda = peripherals.PIN_4;
    let scl = peripherals.PIN_5;

    let mut config = i2c_slave::Config::default();
    config.addr = ADDRESS as u16;
    let device = i2c_slave::I2cSlave::new(peripherals.I2C0, scl, sda, Irqs, config);

    unwrap!(spawner.spawn(i2c_task(device)));

}

#[embassy_executor::task]
async fn i2c_task(mut device: i2c_slave::I2cSlave<'static, I2C0>) {
    let mut io_state = 0b0000_1111;
    let mut matrix_state = 0b0000_1111;

    loop {
        let mut buf = [0u8; 128];
        match device.listen(&mut buf).await {
            Ok(cmd) => {
                LED.signal(());
                match cmd {
                    i2c_slave::Command::Read => {
                        info!("Read");
                        if let Err(e) = device.respond_to_read(&[io_state]).await {
                            error!("Error: {:?}", e);
                        }
                    }
                    cmd => {
                        info!("Command: {:?}", cmd);
                    }
                }
            }
            Err(e) => {
                error!("Error: {:?}", e);
            }
        }
    }

}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static, PIN_22>) {
    led.set_high();
    loop {
        LED.wait().await;
        led.set_low();
        Timer::after_millis(100).await;
        led.set_high();
        Timer::after_millis(100).await;
    }
}
