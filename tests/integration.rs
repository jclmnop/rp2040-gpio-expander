#![no_std]
#![no_main]

use embassy_rp;
use rp_2040_gpio_expander;

#[defmt_test::tests]
mod tests {
    use super::rp_2040_gpio_expander::prelude::*;
    use super::*;
    use defmt::{assert, assert_eq, info, unwrap};
    use embassy_rp::Peripherals;
    use rp_2040_gpio_expander::device::Device;
    use rp_2040_gpio_expander::gpios::{PinGroup0, PinGroup1};

    struct State {
        pub device: Device,
    }

    #[init]
    fn init() -> State {
        let peripherals = embassy_rp::init(Default::default());
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
        State { device }
    }

    #[test]
    fn commands_parse_correctly() {
        use rp_2040_gpio_expander::commands::{Error, GpioCommand};

        let valid_test_cases = [
            ([0x01, 0, 0], GpioCommand::ReadIoModes),
            ([0x02, 3, 12], GpioCommand::WriteOutputs(3, 12)),
            ([0x03, 4, 255], GpioCommand::SetIoModes(4, 255)),
        ];

        for (input, expected) in valid_test_cases.iter() {
            let result = GpioCommand::from_bytes(input);
            match result {
                Ok(cmd) => assert_eq!(cmd, *expected),
                Err(e) => panic!("Error parsing command: {:?}", e),
            }
        }

        let invalid_test_cases = [[0x04, 0, 0], [0x05, 0, 0], [0x06, 0, 0]];

        for input in invalid_test_cases.iter() {
            let result = GpioCommand::from_bytes(input);
            match result {
                Ok(cmd) => panic!("Expected error, got: {:?}", cmd),
                Err(e) => {
                    info!("{}", e);
                    assert_eq!(e, Error::BadInput);
                }
            }
        }
    }

    #[test]
    fn device_handles_command_correctly(state: &mut State) {
        use rp_2040_gpio_expander::device::Error;
        let write_buf = [0u8; 2];
        let mut read_buf = [0u8; 2];

        state.device.set_pin_modes(&write_buf);
        state.device.write(&write_buf);
        state.device.read(&mut read_buf);
        assert_eq!([255, 255], read_buf);
        state.device.get_pin_modes(&mut read_buf);
        assert_eq!(write_buf, read_buf);

        let pin_mode_test_cases = [
            [0x03, 0b0000_0000, 0b0000_1111],
            [0x03, 0b1010_1010, 0b0101_0101],
            [0x03, 0b0101_0101, 0b0101_0101],
            [0x03, 0b0000_1111, 0b0000_1111], // Set half of the pins to output at the end
        ];

        for cmd in pin_mode_test_cases.iter() {
            unwrap!(state.device.handle_write_command(cmd));
            unwrap!(state
                .device
                .handle_write_read_command(&[0x01], &mut read_buf));
            assert_eq!(cmd[1..], read_buf[..]);
        }

        // (cmd_bytes, expected_pin_states)
        let pin_state_test_cases = [
            ([0x02, 0b0000_0000, 0b0000_1111], [0b0000_0000, 0b1111_1111]),
            ([0x02, 0b0000_1111, 0b0000_0000], [0b1111_1111, 0b0000_0000]),
            ([0x02, 0b1111_0000, 0b1111_0000], [0b0000_0000, 0b0000_0000]),
        ];

        for (cmd, expected_state) in pin_state_test_cases.iter() {
            unwrap!(state.device.handle_write_command(cmd));
            state.device.read(&mut read_buf);
            assert_eq!(expected_state, &read_buf);
        }

        let invalid_test_cases = [
            [0x04, 0b0000_0000, 0b0000_1111],
            [0x05, 0b1010_1010, 0b0101_0101],
            [0x06, 0b0101_0101, 0b0101_0101],
        ];

        for cmd in invalid_test_cases.iter() {
            let result = state.device.handle_write_command(cmd);
            match result {
                Ok(_) => panic!("Expected error, got: {:?}", cmd),
                Err(e) => {
                    info!("{}", e);
                    match e {
                        Error::FailedToParseCmd(_) => {}
                        _ => panic!("Expected FailedToParseCommand, got: {:?}", e),
                    }
                }
            }
        }

        let invalid_write_read_test_cases = [
            [0x02, 0b0000_0000, 0b0000_0000],
            [0x03, 0b0000_0000, 0b0000_0000],
        ];

        for cmd in invalid_write_read_test_cases.iter() {
            let result = state.device.handle_write_read_command(cmd, &mut read_buf);
            match result {
                Ok(_) => panic!("Expected error, got: {:?}", cmd),
                Err(e) => {
                    info!("{}", e);
                    match e {
                        Error::InvalidWriteReadCmd(_) => {}
                        _ => panic!("Expected InvalidWriteReadCmd, got: {:?}", e),
                    }
                }
            }
        }

        let result = state.device.handle_write_command(&[0x01]);
        match result {
            Ok(_) => panic!("Expected error, got: {:?}", 0x01),
            Err(e) => {
                info!("{}", e);
                match e {
                    Error::InvalidWriteCmd(_) => {}
                    _ => panic!("Expected InvalidReadCmd, got: {:?}", e),
                }
            }
        }
    }

    #[test]
    fn outputs_inputs_work(state: &mut State) {
        let mut buf = [0u8; 2];
        state.device.set_pin_modes(&[0b0000_1111, 0b0000_1111]);
        state.device.read(&mut buf);
        info!("GPIO_STATE: {=[u8;2]:08b}\n", &buf);
        state.device.write(&[0b0000_0000, 0b0000_0000]);
        state.device.read(&mut buf);
        info!("GPIO_STATE: {=[u8;2]:08b}\n", &buf);

        for i in 0..4 {
            let write = [0b0000_0001 << i, 0b0000_0000];
            let expected = [write[0] | (0b0000_0001 << i + 4), 0b0000_0000];
            info!("Writing: {=[u8;2]:08b}", &write);
            state.device.write(&write);
            state.device.read(&mut buf);
            info!(
                "GPIO_STATE: {=[u8;2]:08b} \tExpected: {=[u8;2]:08b}\n",
                &buf, &expected
            );
            assert_eq!(buf, expected);

            let write = [0b0000_0000, 0b0000_0001 << i];
            let expected = [0b0000_0000, write[1] | (0b0000_0001 << i + 4)];
            info!("Writing: {=[u8;2]:08b}", &write);
            state.device.write(&write);
            state.device.read(&mut buf);
            info!(
                "GPIO_STATE: {=[u8;2]:08b} \tExpected: {=[u8;2]:08b}\n",
                &buf, &expected
            );
            assert_eq!(buf, expected);
        }

        info!("Swapping inputs and outputs");
        state.device.set_pin_modes(&[0b1111_0000, 0b1111_0000]);
        state.device.write(&[0b0000_0000, 0b0000_0000]);
        state.device.read(&mut buf);
        info!("GPIO_STATE: {=[u8;2]:08b}\n", &buf);

        for i in 4..8 {
            let write = [0b0000_0001 << i, 0b0000_0000];
            let expected = [write[0] | (0b0000_0001 << i - 4), 0b0000_0000];
            info!("Writing: {=[u8;2]:08b}", &write);
            state.device.write(&write);
            state.device.read(&mut buf);
            info!(
                "GPIO_STATE: {=[u8;2]:08b} \tExpected: {=[u8;2]:08b}\n",
                &buf, &expected
            );
            assert_eq!(buf, expected);

            let write = [0b0000_0000, 0b0000_0001 << i];
            let expected = [0b0000_0000, (write[1] | 0b0000_0001 << i - 4)];
            info!("Writing: {=[u8;2]:08b}", &write);
            state.device.write(&write);
            state.device.read(&mut buf);
            info!(
                "GPIO_STATE: {=[u8;2]:08b} \tExpected: {=[u8;2]:08b}\n",
                &buf, &expected
            );
            assert_eq!(buf, expected);
        }
    }
}
