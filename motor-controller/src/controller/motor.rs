use esp_idf_svc::hal::{
    gpio::{Output, Pin, PinDriver},
    ledc::LedcDriver,
};

use super::job::MotorDirection;

/// Represents a Motor configuration with a L298N chip
pub struct Motor<'a, InAPin: Pin, InBPin: Pin> {
    pub in_a_pin: PinDriver<'a, InAPin, Output>,
    pub in_b_pin: PinDriver<'a, InBPin, Output>,
    pub pwm_pin: LedcDriver<'a>,
}

impl<'a, InAPin: Pin, InBPin: Pin> Motor<'a, InAPin, InBPin> {
    pub const fn new(
        in_a_pin: PinDriver<'a, InAPin, Output>,
        in_b_pin: PinDriver<'a, InBPin, Output>,
        pwm_pin: LedcDriver<'a>,
    ) -> Self {
        Self {
            in_a_pin,
            in_b_pin,
            pwm_pin,
        }
    }

    /// Set the motor to a specific power level
    /// The power level is a value between 0 and 100
    /// 0 means no power, 100 means full power
    /// The direction is determined by the in_a_pin and in_b_pin
    pub fn drive_motor(&mut self, direction: MotorDirection, power: u8) {
        if power == 0 {
            self.in_a_pin.set_low().unwrap();
            self.in_b_pin.set_low().unwrap();
            self.pwm_pin.set_duty(0).unwrap();
        } else {
            match direction {
                MotorDirection::Forward => {
                    self.in_a_pin.set_high().unwrap();
                    self.in_b_pin.set_low().unwrap();
                }
                MotorDirection::Backward => {
                    self.in_a_pin.set_low().unwrap();
                    self.in_b_pin.set_high().unwrap();
                }
            }
            let power = (power.min(100) as f32 / 100.0 * self.pwm_pin.get_max_duty() as f32) as u32;
            self.pwm_pin.set_duty(power).unwrap();
        }
    }
}

/// An abstract representation of a motor
/// This trait is used to allow any motor driver to be used with the motor controller
pub trait AnyMotor {
    fn drive_motor(&mut self, direction: MotorDirection, power: u8);
}

impl<InAPin: Pin, InBPin: Pin> AnyMotor for Motor<'_, InAPin, InBPin> {
    fn drive_motor(&mut self, direction: MotorDirection, power: u8) {
        Motor::drive_motor(self, direction, power);
    }
}
