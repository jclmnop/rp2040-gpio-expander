#![no_std]
#![no_main]

use defmt::*;
use device::Device;
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Flex, Level, Output, Pin, Pull};
use embassy_rp::i2c_slave::Command;
use embassy_rp::interrupt::{InterruptExt, Priority};
use embassy_rp::peripherals::{I2C0, PIN_22, PIN_26};
use embassy_rp::{bind_interrupts, i2c, i2c_slave, interrupt};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use gpios::{PinGroup0, PinGroup1};
use rp_2040_gpio_expander::prelude::*;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

#[allow(non_camel_case_types)]
type P_INT_OUT = PIN_26;

const ADDRESS: u8 = 0x20;
const DEFAULT_PIN_MODES: [u8; 2] = [0b0000_0001, 0b1111_0000];
static EXECUTOR_HIGH: InterruptExecutor = InterruptExecutor::new();
static LED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

//TODO: try without main macro to make sure multiprio executors are working properly
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let int_out: P_INT_OUT = peripherals.PIN_26;
    interrupt::SWI_IRQ_0.set_priority(Priority::P2);
    let high_spawner = EXECUTOR_HIGH.start(interrupt::SWI_IRQ_0);
    unwrap!(high_spawner.spawn(trigger_int_out(int_out)));

    let led = Output::new(peripherals.PIN_22, Level::Low);

    unwrap!(spawner.spawn(led_task(led)));

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

    unwrap!(spawner.spawn(i2c_task(slave, device)));
}

#[interrupt]
unsafe fn SWI_IRQ_0() {
    EXECUTOR_HIGH.on_interrupt();
}

#[embassy_executor::task]
async fn trigger_int_out(int_out: P_INT_OUT) {
    let mut int_out = Flex::new(int_out);
    int_out.set_pull(Pull::Up);
    int_out.set_as_output();
    int_out.set_level(false.into());
    loop {
        let level = SET_INT_OUT.wait().await;
        int_out.set_level(level.into());
        info!("[INT_OUT] LEVEL: {}", level);
    }
}

#[embassy_executor::task]
async fn i2c_task(mut slave: i2c_slave::I2cSlave<'static, I2C0>, mut device: Device) {
    let mut write_buf = [0u8; 128];
    let mut read_buf = [0u8; 2];

    device.set_pin_modes(&DEFAULT_PIN_MODES);
    device.read(&mut read_buf);

    //TODO: move to `run` method for easier testing
    info!("[MAIN_TASK] STARTING");
    // info!("[MAIN_TASK] GPIO_STATE: {=[u8;2]:08b}", &read_buf);
    loop {
        write_buf.fill(0);
        // read_buf.fill(0);
        device.read(&mut read_buf);
        info!("[MAIN_TASK] GPIO_STATE: {=[u8;2]:08b}", &read_buf);
        match select(device.wait_for_any_edge(), slave.listen(&mut write_buf)).await {
            Either::First(_) => {
                // device.read(&mut read_buf);
                // info!("[MAIN_TASK] GPIO_STATE: {=[u8;2]:08b}", &read_buf);
            }
            Either::Second(listen_result) => {
                // LED.signal(());
                match listen_result {
                    Ok(Command::GeneralCall(_)) => {
                        info!("[MAIN_TASK] GENERAL CALL");
                    }
                    Ok(Command::Read) => {
                        info!("[MAIN_TASK] READ");
                        device.handle_read_command(&mut read_buf);
                        match slave.respond_and_fill(&read_buf, 0x00).await {
                            Ok(read_status) => {
                                info!("[MAIN_TASK] READ_RESPONSE: {:?}", &read_buf);
                                info!("[MAIN_TASK] READ_STATUS: {:?}", read_status);
                            }
                            Err(e) => {
                                error!("[MAIN_TASK] READ_RESPONSE: {}", e);
                            }
                        }
                    }
                    Ok(Command::Write(len)) => {
                        info!("[MAIN_TASK] WRITE: {:?}", &write_buf[..len]);
                        if let Err(e) = device.handle_write_command(&write_buf[..len]) {
                            error!("[MAIN_TASK] WRITE_ERROR: {:?}", e);
                        }
                    }
                    Ok(Command::WriteRead(len)) => {
                        info!("[MAIN_TASK] WRITE_READ: {:?}", &write_buf[..len]);
                        if let Err(e) =
                            device.handle_write_read_command(&write_buf[..len], &mut read_buf)
                        {
                            error!("[MAIN_TASK] WRITE_READ_ERROR: {:?}", e);
                        } else {
                            match slave.respond_and_fill(&read_buf, 0x00).await {
                                Ok(read_status) => {
                                    info!("[MAIN_TASK] WRITE_READ_RESPONSE: {:?}", &read_buf);
                                    info!("[MAIN_TASK] WRITE_READ_STATUS: {:?}", read_status);
                                }
                                Err(e) => {
                                    error!("[MAIN_TASK] WRITE_READ_RESPONSE: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("[MAIN_TASK] LISTEN_ERROR: {:#?}", e);
                    }
                }
            }
        }
        // match i2c_slave.listen(&mut write_buf).await {
        //     Ok(cmd) => {
        //         LED.signal(());
        //         match cmd {
        //             i2c_slave::Command::Read => {
        //                 info!("Read");
        //                 if let Err(e) = i2c_slave.respond_to_read(&[io_state]).await {
        //                     error!("Error: {:?}", e);
        //                 }
        //             }
        //             cmd => {
        //                 info!("Command: {:?}", cmd);
        //             }
        //         }
        //     }
        //     Err(e) => {
        //         error!("Error: {:?}", e);
        //     }
        // }
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
