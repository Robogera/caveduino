#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use core::intrinsics::unchecked_add;

use arduino_hal::{
    delay_ms,
    prelude::*,
    simple_pwm::{IntoPwmPin, *},
};
use panic_halt as _;

use debouncr::{debounce_stateful_16, Edge};

const BRGHT_DEPTH: u8 = 125;
const BRGHT_STEP: u8 = 1;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let timer0 = Timer0Pwm::new(dp.TC0, Prescaler::Prescale64);

    let mut led1_pwm = pins.d5.into_output().into_pwm(&timer0);
    let mut led2_pwm = pins.d6.into_output().into_pwm(&timer0);
    led1_pwm.enable();
    led2_pwm.enable();

    let butt1 = pins.a1.into_pull_up_input();
    let butt2 = pins.a3.into_pull_up_input();

    let mut db1 = debounce_stateful_16(true);
    let mut db2 = debounce_stateful_16(true);

    let mut brght = 0_u8;
    let mut brght_up = true;

    loop {
        let edge1 = db1.update(butt1.is_low());
        let edge2 = db2.update(butt2.is_low());
        if let Some(Edge::Falling) = edge1 {
            ufmt::uwriteln!(&mut serial, "{}", 'n').unwrap_infallible();
        }
        if let Some(Edge::Falling) = edge2 {
            ufmt::uwriteln!(&mut serial, "{}", 'b').unwrap_infallible();
        }

        if brght > 245 {
            brght = 245;
            brght_up = false;
        } else if brght <= 245 - BRGHT_DEPTH {
            brght = 245 - BRGHT_DEPTH;
            brght_up = true;
        }
        led1_pwm.set_duty(brght);
        led2_pwm.set_duty(brght);
        brght = if brght_up {
            brght + BRGHT_STEP
        } else {
            brght - BRGHT_STEP
        };
        delay_ms(10);
    }
}
