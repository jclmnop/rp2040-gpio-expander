use crate::commands::GpioCommand;
use crate::gpios::{PinGroup0, PinGroup1};
use crate::SET_INT_OUT;
use defmt::{info, Format};
use embassy_rp::gpio::Pull;
// use embassy_futures::yield_now;

pub struct Device {
    pub gpio_group_0: PinGroup0,
    pub gpio_group_1: PinGroup1,
}

impl Device {
    pub fn new(gpio_group_0: PinGroup0, gpio_group_1: PinGroup1) -> Self {
        Self {
            gpio_group_0,
            gpio_group_1,
        }
    }
}

/// Pin related methods
impl Device {
    pub fn read(&self, out: &mut [u8; 2]) {
        out[0] = self.gpio_group_0.read_pins();
        out[1] = self.gpio_group_1.read_pins();
    }

    pub fn write(&mut self, bytes: &[u8; 2]) {
        self.gpio_group_0.write_pins(bytes[0]);
        self.gpio_group_1.write_pins(bytes[1]);
    }

    pub fn write1(&mut self, byte: u8) {
        self.gpio_group_0.write_pins(byte);
    }

    pub fn write2(&mut self, byte: u8) {
        self.gpio_group_1.write_pins(byte);
    }

    pub fn set_pin_modes(&mut self, bytes: &[u8; 2]) {
        self.gpio_group_0.set_pin_modes(bytes[0]);
        self.gpio_group_1.set_pin_modes(bytes[1]);
    }

    pub fn get_pin_modes(&self, out: &mut [u8; 2]) {
        out[0] = self.gpio_group_0.get_pin_modes();
        out[1] = self.gpio_group_1.get_pin_modes();
    }

    pub fn set_pin_pulls(&mut self, bytes: &[u8; 2], pull: Pull) {
        self.gpio_group_0.set_pin_pulls(bytes[0], pull);
        self.gpio_group_1.set_pin_pulls(bytes[1], pull);
    }

    pub async fn wait_for_any_edge(&mut self) {
        self.gpio_group_0.wait_for_any_edge().await;
        info!("INTERRUPT!");
        // SET_INT_OUT.signal(true);
        // yield_now().await;
    }
}

/// I2C functionality
impl Device {
    pub fn handle_write_command(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let command = GpioCommand::from_bytes(bytes)?;
        info!("Command: {:?}", command);
        match command {
            GpioCommand::WriteAllOutputs(gpio_group_0, gpio_group_1) => {
                Ok(self.write(&[gpio_group_0, gpio_group_1]))
            }
            GpioCommand::SetIoModes(gpio_group_0, gpio_group_1) => {
                Ok(self.set_pin_modes(&[gpio_group_0, gpio_group_1]))
            }
            GpioCommand::WriteOutputs1(gpio_group_1) => Ok(self.write1(gpio_group_1)),
            GpioCommand::WriteOutputs2(gpio_group_2) => Ok(self.write2(gpio_group_2)),
            GpioCommand::SetPullDowns(gpio_group1, gpio_group_2) => {
                Ok(self.set_pin_pulls(&[gpio_group1, gpio_group_2], Pull::Down))
            }
            GpioCommand::SetPullUps(gpio_group1, gpio_group_2) => {
                Ok(self.set_pin_pulls(&[gpio_group1, gpio_group_2], Pull::Up))
            }
            GpioCommand::SetPullNone(gpio_group1, gpio_group_2) => {
                Ok(self.set_pin_pulls(&[gpio_group1, gpio_group_2], Pull::None))
            }
            otherwise => Err(Error::InvalidWriteCmd(otherwise)),
        }
    }

    pub fn handle_write_read_command(
        &mut self,
        bytes: &[u8],
        out: &mut [u8; 2],
    ) -> Result<usize, Error> {
        let command = GpioCommand::from_bytes(bytes)?;
        match command {
            GpioCommand::ReadIoModes => {
                self.get_pin_modes(out);
                Ok(2)
            }
            GpioCommand::ReadInputs1 => {
                out[0] = self.gpio_group_0.read_pins();
                self.gpio_group_0.clear_int_out();
                Ok(1)
            }
            GpioCommand::ReadInputs2 => {
                out[0] = self.gpio_group_1.read_pins();
                Ok(1)
            }
            otherwise => Err(Error::InvalidWriteReadCmd(otherwise)),
        }
    }

    pub fn handle_read_command(&self, out: &mut [u8; 2]) {
        SET_INT_OUT.signal(false);
        self.read(out);
    }
}

#[derive(Debug, Clone, Copy, Format, Eq, PartialEq)]
pub enum Error {
    FailedToParseCmd(crate::commands::Error),
    InvalidWriteCmd(GpioCommand),
    InvalidWriteReadCmd(GpioCommand),
}

impl From<crate::commands::Error> for Error {
    fn from(err: crate::commands::Error) -> Self {
        Self::FailedToParseCmd(err)
    }
}
