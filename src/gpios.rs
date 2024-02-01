//! TODO: might be more efficient to use DMA but this is easier for now
use embassy_rp::gpio::{Flex, Pin, Pull};
use embassy_rp::peripherals::*;

pub type PinGroup0 = PinGroup<PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13>;

pub type PinGroup1 = PinGroup<PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21>;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PinMask {
    P0 = 0b0000_0001,
    P1 = 0b0000_0010,
    P2 = 0b0000_0100,
    P3 = 0b0000_1000,
    P4 = 0b0001_0000,
    P5 = 0b0010_0000,
    P6 = 0b0100_0000,
    P7 = 0b1000_0000,
}

impl PinMask {
    pub const fn to_u8(&self) -> u8 {
        *self as u8
    }
}

//TODO: wait for any_edge using select
/// Groups 8 pins together so that they can be read from, and written to, as a single byte
pub struct PinGroup<P0: Pin, P1: Pin, P2: Pin, P3: Pin, P4: Pin, P5: Pin, P6: Pin, P7: Pin> {
    p0: Flex<'static, P0>,
    p1: Flex<'static, P1>,
    p2: Flex<'static, P2>,
    p3: Flex<'static, P3>,
    p4: Flex<'static, P4>,
    p5: Flex<'static, P5>,
    p6: Flex<'static, P6>,
    p7: Flex<'static, P7>,
    pin_modes: u8,
}

impl<P0: Pin, P1: Pin, P2: Pin, P3: Pin, P4: Pin, P5: Pin, P6: Pin, P7: Pin>
    PinGroup<P0, P1, P2, P3, P4, P5, P6, P7>
{
    pub fn new(p0: P0, p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7) -> Self {
        let mut this = Self {
            p0: Flex::new(p0),
            p1: Flex::new(p1),
            p2: Flex::new(p2),
            p3: Flex::new(p3),
            p4: Flex::new(p4),
            p5: Flex::new(p5),
            p6: Flex::new(p6),
            p7: Flex::new(p7),
            pin_modes: 0,
        };
        this.set_pin_modes(0); // Initially set all pins to input mode
        this
    }

    pub fn set_pin_modes(&mut self, bits: u8) {
        self.set_pin_mode(bits, &PinMask::P0);
        self.set_pin_mode(bits, &PinMask::P1);
        self.set_pin_mode(bits, &PinMask::P2);
        self.set_pin_mode(bits, &PinMask::P3);
        self.set_pin_mode(bits, &PinMask::P4);
        self.set_pin_mode(bits, &PinMask::P5);
        self.set_pin_mode(bits, &PinMask::P6);
        self.set_pin_mode(bits, &PinMask::P7);
        self.pin_modes = bits;
    }

    pub fn get_pin_modes(&self) -> u8 {
        self.pin_modes
    }

    pub fn set_pin_mode(&mut self, bits: u8, pin_mask: &PinMask) {
        if bits & pin_mask.to_u8() == pin_mask.to_u8() {
            self.set_pin_output(pin_mask);
        } else {
            self.set_pin_input(pin_mask);
        }
    }

    pub fn set_pin_output(&mut self, pin_mask: &PinMask) {
        match pin_mask {
            PinMask::P0 => self.p0.set_as_output(),
            PinMask::P1 => self.p1.set_as_output(),
            PinMask::P2 => self.p2.set_as_output(),
            PinMask::P3 => self.p3.set_as_output(),
            PinMask::P4 => self.p4.set_as_output(),
            PinMask::P5 => self.p5.set_as_output(),
            PinMask::P6 => self.p6.set_as_output(),
            PinMask::P7 => self.p7.set_as_output(),
        }
    }

    pub fn set_pin_input(&mut self, pin_mask: &PinMask) {
        match pin_mask {
            PinMask::P0 => self.p0.set_as_input(),
            PinMask::P1 => self.p1.set_as_input(),
            PinMask::P2 => self.p2.set_as_input(),
            PinMask::P3 => self.p3.set_as_input(),
            PinMask::P4 => self.p4.set_as_input(),
            PinMask::P5 => self.p5.set_as_input(),
            PinMask::P6 => self.p6.set_as_input(),
            PinMask::P7 => self.p7.set_as_input(),
        }
        //TODO: configurable pull up/down
        match pin_mask {
            PinMask::P0 => self.p0.set_pull(Pull::Up),
            PinMask::P1 => self.p1.set_pull(Pull::Up),
            PinMask::P2 => self.p2.set_pull(Pull::Up),
            PinMask::P3 => self.p3.set_pull(Pull::Up),
            PinMask::P4 => self.p4.set_pull(Pull::Up),
            PinMask::P5 => self.p5.set_pull(Pull::Up),
            PinMask::P6 => self.p6.set_pull(Pull::Up),
            PinMask::P7 => self.p7.set_pull(Pull::Up),
        }
    }

    pub fn is_pin_output(&self, pin_mask: &PinMask) -> bool {
        self.pin_modes & pin_mask.to_u8() == pin_mask.to_u8()
    }

    pub fn write_pins(&mut self, byte: u8) {
        self.write_pin(
            &PinMask::P0,
            byte & PinMask::P0.to_u8() == PinMask::P0.to_u8(),
        );
        self.write_pin(
            &PinMask::P1,
            byte & PinMask::P1.to_u8() == PinMask::P1.to_u8(),
        );
        self.write_pin(
            &PinMask::P2,
            byte & PinMask::P2.to_u8() == PinMask::P2.to_u8(),
        );
        self.write_pin(
            &PinMask::P3,
            byte & PinMask::P3.to_u8() == PinMask::P3.to_u8(),
        );
        self.write_pin(
            &PinMask::P4,
            byte & PinMask::P4.to_u8() == PinMask::P4.to_u8(),
        );
        self.write_pin(
            &PinMask::P5,
            byte & PinMask::P5.to_u8() == PinMask::P5.to_u8(),
        );
        self.write_pin(
            &PinMask::P6,
            byte & PinMask::P6.to_u8() == PinMask::P6.to_u8(),
        );
        self.write_pin(
            &PinMask::P7,
            byte & PinMask::P7.to_u8() == PinMask::P7.to_u8(),
        );
    }

    pub fn write_pin(&mut self, pin_mask: &PinMask, high: bool) {
        if self.is_pin_output(pin_mask) {
            self.write_output_pin(pin_mask, high);
        }
    }

    fn write_output_pin(&mut self, pin_mask: &PinMask, high: bool) {
        if high {
            match pin_mask {
                PinMask::P0 => self.p0.set_high(),
                PinMask::P1 => self.p1.set_high(),
                PinMask::P2 => self.p2.set_high(),
                PinMask::P3 => self.p3.set_high(),
                PinMask::P4 => self.p4.set_high(),
                PinMask::P5 => self.p5.set_high(),
                PinMask::P6 => self.p6.set_high(),
                PinMask::P7 => self.p7.set_high(),
            }
        } else {
            match pin_mask {
                PinMask::P0 => self.p0.set_low(),
                PinMask::P1 => self.p1.set_low(),
                PinMask::P2 => self.p2.set_low(),
                PinMask::P3 => self.p3.set_low(),
                PinMask::P4 => self.p4.set_low(),
                PinMask::P5 => self.p5.set_low(),
                PinMask::P6 => self.p6.set_low(),
                PinMask::P7 => self.p7.set_low(),
            }
        }
    }

    pub fn read_pins(&self) -> u8 {
        let mut result = 0;

        if self.read_pin(&PinMask::P0) {
            result |= PinMask::P0.to_u8();
        }
        if self.read_pin(&PinMask::P1) {
            result |= PinMask::P1.to_u8();
        }
        if self.read_pin(&PinMask::P2) {
            result |= PinMask::P2.to_u8();
        }
        if self.read_pin(&PinMask::P3) {
            result |= PinMask::P3.to_u8();
        }
        if self.read_pin(&PinMask::P4) {
            result |= PinMask::P4.to_u8();
        }
        if self.read_pin(&PinMask::P5) {
            result |= PinMask::P5.to_u8();
        }
        if self.read_pin(&PinMask::P6) {
            result |= PinMask::P6.to_u8();
        }
        if self.read_pin(&PinMask::P7) {
            result |= PinMask::P7.to_u8();
        }

        result
    }

    pub fn read_pin(&self, pin_mask: &PinMask) -> bool {
        if self.is_pin_output(pin_mask) {
            self.read_output_pin(pin_mask)
        } else {
            self.read_input_pin(pin_mask)
        }
    }

    fn read_output_pin(&self, pin_mask: &PinMask) -> bool {
        match pin_mask {
            PinMask::P0 => self.p0.is_set_high(),
            PinMask::P1 => self.p1.is_set_high(),
            PinMask::P2 => self.p2.is_set_high(),
            PinMask::P3 => self.p3.is_set_high(),
            PinMask::P4 => self.p4.is_set_high(),
            PinMask::P5 => self.p5.is_set_high(),
            PinMask::P6 => self.p6.is_set_high(),
            PinMask::P7 => self.p7.is_set_high(),
        }
    }

    fn read_input_pin(&self, pin_mask: &PinMask) -> bool {
        match pin_mask {
            PinMask::P0 => self.p0.is_high(),
            PinMask::P1 => self.p1.is_high(),
            PinMask::P2 => self.p2.is_high(),
            PinMask::P3 => self.p3.is_high(),
            PinMask::P4 => self.p4.is_high(),
            PinMask::P5 => self.p5.is_high(),
            PinMask::P6 => self.p6.is_high(),
            PinMask::P7 => self.p7.is_high(),
        }
    }
}

pub mod interrupts {
    use super::*;
    use embassy_futures::select::{select, select4};

    impl<P0: Pin, P1: Pin, P2: Pin, P3: Pin, P4: Pin, P5: Pin, P6: Pin, P7: Pin>
        PinGroup<P0, P1, P2, P3, P4, P5, P6, P7>
    {
        pub async fn wait_for_any_edge(&mut self) {
            select(
                select4(
                    self.p0.wait_for_any_edge(),
                    self.p1.wait_for_any_edge(),
                    self.p2.wait_for_any_edge(),
                    self.p3.wait_for_any_edge(),
                ),
                select4(
                    self.p4.wait_for_any_edge(),
                    self.p5.wait_for_any_edge(),
                    self.p6.wait_for_any_edge(),
                    self.p7.wait_for_any_edge(),
                ),
            )
            .await;
        }
    }
}
