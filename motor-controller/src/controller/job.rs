//! Job Models

use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum MotorDirection {
    #[serde(rename = "f")]
    Forward,
    #[serde(rename = "b")]
    Backward,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct MotorJob {
    pub motor_index: usize,
    pub direction: MotorDirection,
    /// A value between 1 and 100
    pub power: u8,
    pub time_ms: u16,
}

impl MotorJob {
    pub fn new(motor_index: usize, direction: MotorDirection, power: u8, time_ms: u16) -> Self {
        Self {
            motor_index,
            direction,
            power,
            time_ms,
        }
    }

    pub fn activate_job(&self) -> ActiveMotorJob {
        ActiveMotorJob {
            current_time_ms: 0,
            job_time_ms: self.time_ms as u32,
        }
    }
}

pub struct ActiveMotorJob {
    pub current_time_ms: u32,
    pub job_time_ms: u32,
}

impl ActiveMotorJob {
    pub fn is_active(&self) -> bool {
        self.current_time_ms < self.job_time_ms
    }

    pub fn increment_time(&mut self, time_ms: u16) {
        self.current_time_ms += time_ms as u32;
    }
}
