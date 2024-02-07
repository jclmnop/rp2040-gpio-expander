use crate::commands::GpioCommand;
use crate::gpios::{PinGroup0, PinGroup1};
use crate::SET_INT_OUT;
use defmt::{info, Format};
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

    pub fn set_pin_modes(&mut self, bytes: &[u8; 2]) {
        self.gpio_group_0.set_pin_modes(bytes[0]);
        self.gpio_group_1.set_pin_modes(bytes[1]);
    }

    pub fn get_pin_modes(&self, out: &mut [u8; 2]) {
        out[0] = self.gpio_group_0.get_pin_modes();
        out[1] = self.gpio_group_1.get_pin_modes();
    }

    pub async fn wait_for_any_edge(&mut self) {
        self.gpio_group_0.wait_for_any_edge().await;
        SET_INT_OUT.signal(true);
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
            otherwise => Err(Error::InvalidWriteCmd(otherwise)),
        }
    }

    pub fn handle_write_read_command(
        &mut self,
        bytes: &[u8],
        out: &mut [u8; 2],
    ) -> Result<(), Error> {
        let command = GpioCommand::from_bytes(bytes)?;
        match command {
            GpioCommand::ReadIoModes => Ok(self.get_pin_modes(out)),
            GpioCommand::ReadInputs1 => {
                out[0] = self.gpio_group_0.read_pins();
                self.gpio_group_0.clear_int_out();
                Ok(())
            }
            GpioCommand::ReadInputs2 => {
                out[0] = self.gpio_group_1.read_pins();
                Ok(())
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
