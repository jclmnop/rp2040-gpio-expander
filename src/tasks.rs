use crate::prelude::*;
use device::Device;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Level, Output, OutputOpenDrain};
use embassy_rp::i2c_slave::Command;
use embassy_rp::peripherals::I2C0;
use embassy_rp::{i2c_slave, interrupt};
use embassy_time::Timer;

const EN_DELAY_MS: u64 = 200;
const EN_DURATION: u64 = 100;

#[interrupt]
unsafe fn SWI_IRQ_0() {
    EXECUTOR_HIGH.on_interrupt();
}

#[embassy_executor::task]
pub async fn trigger_int_out(int_out: P_INT_OUT) -> ! {
    let mut int_out = OutputOpenDrain::new(int_out, Level::Low);
    loop {
        let level = SET_INT_OUT.wait().await;
        int_out.set_level((!level).into());
        info!("[INT_OUT] LEVEL: {}", level);
    }
}

#[embassy_executor::task]
pub async fn i2c_task(mut slave: i2c_slave::I2cSlave<'static, I2C0>, mut device: Device) -> ! {
    let mut write_buf = [0u8; 128];
    let mut read_buf = [0u8; 2];

    device.set_pin_modes(&DEFAULT_PIN_MODES);
    device.read(&mut read_buf);

    info!("[MAIN_TASK] STARTING");
    // info!("[MAIN_TASK] GPIO_STATE: {=[u8;2]:08b}", &read_buf);
    loop {
        write_buf.fill(0);
        device.read(&mut read_buf);
        info!("[MAIN_TASK] GPIO_STATE: {=[u8;2]:08b}", &read_buf);
        read_buf.fill(0);
        match select(device.wait_for_any_edge(), slave.listen(&mut write_buf)).await {
            Either::First(_) => {
                //TODO: move back to wait_for_any_edge()?
                SET_INT_OUT.signal(true);
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
                        match device.handle_write_read_command(&write_buf[..len], &mut read_buf) {
                            Err(e) => {
                                error!("[MAIN_TASK] WRITE_READ_ERROR: {:?}", e);
                            }
                            Ok(out_len) => {
                                match slave.respond_and_fill(&read_buf[..out_len], 0x00).await {
                                    // TODO: fix
                                    Ok(read_status) => {
                                        info!(
                                            "[MAIN_TASK] WRITE_READ_RESPONSE: {:?}",
                                            &read_buf[..out_len]
                                        );
                                        info!("[MAIN_TASK] WRITE_READ_STATUS: {:?}", read_status);
                                    }
                                    Err(e) => {
                                        error!("[MAIN_TASK] WRITE_READ_RESPONSE: {}", e);
                                    }
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
pub async fn led_task(mut led: Output<'static, P_LED>) -> ! {
    led.set_high();
    loop {
        LED.wait().await;
        led.set_low();
        Timer::after_millis(100).await;
        led.set_high();
        Timer::after_millis(100).await;
    }
}

/// Trigger the EN_OUT pin on power up to reboot main board
#[embassy_executor::task]
pub async fn trigger_en_out(en_out: P_EN_OUT) {
    let mut en_out = OutputOpenDrain::new(en_out, Level::Low);
    Timer::after_millis(EN_DELAY_MS).await;
    en_out.set_level(Level::High);
    Timer::after_millis(EN_DURATION).await;
    en_out.set_level(Level::Low);
}
