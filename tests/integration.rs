#![no_std]
#![no_main]

use embassy_rp;
use rp_2040_gpio_expander;

#[defmt_test::tests]
mod tests {
    use super::rp_2040_gpio_expander::prelude::*;
    use super::*;
    use defmt::{assert, info, unwrap};
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
    fn it_works() {
        assert!(true)
    }

    #[test]
    fn it_works_with_state(state: &mut State) {
        let mut buf = [0u8; 2];
        state.device.read(&mut buf);
        info!("GPIO_STATE: {=[u8;2]:08b}", &buf);
        assert!(true)
    }
}
