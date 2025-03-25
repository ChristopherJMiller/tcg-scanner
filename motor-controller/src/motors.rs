use esp_idf_svc::hal::gpio::{Gpio16, Gpio17, Gpio18, Gpio19, Gpio4, Gpio5};

use crate::controller::{
    motor::{AnyMotor, Motor},
    MotorContainer,
};

pub enum TcgMotor<'a> {
    TopDrawer0(Motor<'a, Gpio19, Gpio18>),
    BackLower1(Motor<'a, Gpio5, Gpio17>),
    FrontLower2(Motor<'a, Gpio16, Gpio4>),
}

impl MotorContainer<'_> for TcgMotor<'_> {
    fn get_motor(&mut self) -> &mut dyn AnyMotor {
        match self {
            TcgMotor::TopDrawer0(motor) => motor,
            TcgMotor::BackLower1(motor) => motor,
            TcgMotor::FrontLower2(motor) => motor,
        }
    }
}
