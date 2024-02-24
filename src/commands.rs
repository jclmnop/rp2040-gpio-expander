use byte::ctx::Endian;
use byte::{BytesExt, TryRead};
use defmt::{error, Format};

#[derive(Debug, Clone, Copy, Format, Eq, PartialEq)]
#[repr(u8)]
pub enum GpioCommand {
    ReadIoModes = 0x01,
    WriteAllOutputs(u8, u8) = 0x02,
    SetIoModes(u8, u8) = 0x03,
    WriteOutputs1(u8) = 0x11,
    WriteOutputs2(u8) = 0x12,
    ReadInputs1 = 0x21,
    ReadInputs2 = 0x22,
    SetPullDowns(u8, u8) = 0x30,
    SetPullUps(u8, u8) = 0x31,
    SetPullNone(u8, u8) = 0x32,
}

impl GpioCommand {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: Only safe if the enum is repr(u8)
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

impl<'a> TryRead<'a, Endian> for GpioCommand {
    fn try_read(bytes: &'a [u8], ctx: Endian) -> byte::Result<(Self, usize)> {
        let mut offset = 0;
        let command_byte = bytes.read_with::<u8>(&mut offset, ctx)?;
        let command = match command_byte {
            cmd if cmd == Self::ReadIoModes.discriminant() => Ok(Self::ReadIoModes),
            cmd_with_args
                if cmd_with_args < Self::WriteOutputs1(0).discriminant()
                    || cmd_with_args >= Self::SetPullDowns(0, 0).discriminant() =>
            {
                let gpio_group_1 = bytes.read_with::<u8>(&mut offset, ctx)?;
                let gpio_group_2 = bytes.read_with::<u8>(&mut offset, ctx)?;
                match cmd_with_args {
                    cmd if cmd == Self::WriteAllOutputs(0, 0).discriminant() => {
                        Ok(Self::WriteAllOutputs(gpio_group_1, gpio_group_2))
                    }
                    cmd if cmd == Self::SetIoModes(0, 0).discriminant() => {
                        Ok(Self::SetIoModes(gpio_group_1, gpio_group_2))
                    }
                    cmd if cmd == Self::SetPullDowns(0, 0).discriminant() => {
                        Ok(Self::SetPullDowns(gpio_group_1, gpio_group_2))
                    }
                    cmd if cmd == Self::SetPullUps(0, 0).discriminant() => {
                        Ok(Self::SetPullUps(gpio_group_1, gpio_group_2))
                    }
                    cmd if cmd == Self::SetPullNone(0, 0).discriminant() => {
                        Ok(Self::SetPullNone(gpio_group_1, gpio_group_2))
                    }
                    otherwise => {
                        error!("Invalid command byte with 2 args: {:x}", otherwise);
                        Err(byte::Error::BadInput {
                            err: "Invalid command byte",
                        })
                    }
                }
            }
            cmd_with_arg if cmd_with_arg < Self::ReadInputs1.discriminant() => {
                let gpio_group = bytes.read_with::<u8>(&mut offset, ctx)?;
                match cmd_with_arg {
                    cmd if cmd == Self::WriteOutputs1(0).discriminant() => {
                        Ok(Self::WriteOutputs1(gpio_group))
                    }
                    cmd if cmd == Self::WriteOutputs2(0).discriminant() => {
                        Ok(Self::WriteOutputs2(gpio_group))
                    }
                    otherwise => {
                        error!("Invalid command byte with 1 arg: {:x}", otherwise);
                        Err(byte::Error::BadInput {
                            err: "Invalid command byte",
                        })
                    }
                }
            }
            cmd if cmd == Self::ReadInputs1.discriminant() => Ok(Self::ReadInputs1),
            cmd if cmd == Self::ReadInputs2.discriminant() => Ok(Self::ReadInputs2),
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
