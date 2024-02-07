use byte::ctx::Endian;
use byte::{BytesExt, TryRead};
use defmt::{error, Format};

#[derive(Debug, Clone, Copy, Format, Eq, PartialEq)]
#[repr(u8)]
pub enum GpioCommand {
    ReadIoModes,
    WriteAllOutputs(u8, u8),
    SetIoModes(u8, u8),
    WriteOutputs1(u8),
    WriteOutputs2(u8),
    ReadInputs1,
    ReadInputs2,
    // SetPullUps(u8, u8),
    // SetPullDowns(u8, u8),
    // SetPullNone(u8, u8),
}

impl<'a> TryRead<'a, Endian> for GpioCommand {
    fn try_read(bytes: &'a [u8], ctx: Endian) -> byte::Result<(Self, usize)> {
        let mut offset = 0;
        let command_byte = bytes.read_with::<u8>(&mut offset, ctx)?;
        let command = match command_byte {
            0x01 => Ok(GpioCommand::ReadIoModes),
            cmd_with_args if cmd_with_args < 0x11 => {
                let gpio_group_1 = bytes.read_with::<u8>(&mut offset, ctx)?;
                let gpio_group_2 = bytes.read_with::<u8>(&mut offset, ctx)?;
                match cmd_with_args {
                    0x02 => Ok(GpioCommand::WriteAllOutputs(gpio_group_1, gpio_group_2)),
                    0x03 => Ok(GpioCommand::SetIoModes(gpio_group_1, gpio_group_2)),
                    otherwise => {
                        error!("Invalid command byte with 2 args: {:x}", otherwise);
                        Err(byte::Error::BadInput {
                            err: "Invalid command byte",
                        })
                    }
                }
            }
            cmd_with_arg if cmd_with_arg < 0x21 => {
                let gpio_group = bytes.read_with::<u8>(&mut offset, ctx)?;
                match cmd_with_arg {
                    0x11 => Ok(GpioCommand::WriteOutputs1(gpio_group)),
                    0x12 => Ok(GpioCommand::WriteOutputs2(gpio_group)),
                    otherwise => {
                        error!("Invalid command byte with 1 arg: {:x}", otherwise);
                        Err(byte::Error::BadInput {
                            err: "Invalid command byte",
                        })
                    }
                }
            }
            0x21 => Ok(GpioCommand::ReadInputs1),
            0x22 => Ok(GpioCommand::ReadInputs2),
            otherwise => {
                error!("Invalid command byte: {:x}", otherwise);
                Err(byte::Error::BadInput {
                    err: "Invalid command byte",
                })
            }
        }?;

        Ok((command, offset))
    }
}

impl GpioCommand {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let (command, _) = Self::try_read(bytes, Endian::default()).map_err(Error::from)?;
        Ok(command)
    }
}

#[derive(Debug, Clone, Copy, Format, Eq, PartialEq)]
pub enum Error {
    BadOffset,
    BadInput,
    Incomplete,
}

impl From<byte::Error> for Error {
    fn from(e: byte::Error) -> Self {
        match e {
            byte::Error::BadOffset { .. } => Error::BadOffset,
            byte::Error::BadInput { .. } => Error::BadInput,
            byte::Error::Incomplete { .. } => Error::Incomplete,
        }
    }
}
