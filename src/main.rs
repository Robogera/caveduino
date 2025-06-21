#![no_std]
#![no_main]

use arduino_hal::{
    delay_ms,
    prelude::*,
    simple_pwm::{IntoPwmPin, *},
};
use panic_halt as _;

use debouncr::{debounce_stateful_16, Edge};

const BR_STEP: u8 = 1;

enum BrightnessMode {
    Up,
    Down,
}

impl BrightnessMode {
    fn flip(&mut self) {
        match self {
            Self::Up => *self = BrightnessMode::Down,
            Self::Down => *self = BrightnessMode::Up,
        };
    }
}

enum ButtonMode {
    Off,
    On,
    StartOnly,
}

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
    let mut brightness_mode = BrightnessMode::Down;
    let mut button_mode = ButtonMode::StartOnly;

    loop {
        let edge1 = db1.update(butt1.is_low());
        let edge2 = db2.update(butt2.is_low());
        if let Some(Edge::Falling) = edge1 {
            serial.write_byte(b'n');
            serial.write_byte(b'\n');
        }
        if let Some(Edge::Falling) = edge2 {
            serial.write_byte(b'p');
            serial.write_byte(b'\n');
        }

        button_mode = match serial.read() {
            Ok(b's') => ButtonMode::StartOnly,
            Ok(b'n') => ButtonMode::Off,
            Ok(b'b') => ButtonMode::On,
            _ => button_mode,
        };

        match button_mode {
            ButtonMode::StartOnly => {
                led2_pwm.set_duty(125 + brght / 2);
                led1_pwm.set_duty(0);
            }
            ButtonMode::On => {
                led1_pwm.set_duty(4 + brght / 10);
                led2_pwm.set_duty(4 + brght / 10);
            }
            ButtonMode::Off => {
                led1_pwm.set_duty(4);
                led2_pwm.set_duty(4);
            }
        }

        brght = match brightness_mode {
            BrightnessMode::Up => match brght.checked_add(BR_STEP) {
                Some(value) => value,
                None => {
                    brightness_mode.flip();
                    core::u8::MAX
                }
            },
            BrightnessMode::Down => match brght.checked_sub(BR_STEP) {
                Some(value) => value,
                None => {
                    brightness_mode.flip();
                    core::u8::MIN
                }
            },
        };
        delay_ms(10);
    }
}
