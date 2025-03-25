//! Motor Controller
//!
//! This module contains a global motor controller that managed registered motors.

pub mod job;
pub mod motor;

use std::collections::VecDeque;

use job::{ActiveMotorJob, MotorDirection, MotorJob};
use motor::AnyMotor;

// Represents a generalized motor driver of any motor
pub trait MotorContainer<'a> {
    fn get_motor(&mut self) -> &mut dyn AnyMotor;
}

/// Managed a series of motor drives via job queues
pub struct MotorController<'a, const MOTOR_COUNT: usize> {
    /// Registered pin drivers
    motor_drivers: [&'a mut dyn MotorContainer<'a>; MOTOR_COUNT],
    /// Work Queues for the drivers
    work_queues: [VecDeque<MotorJob>; MOTOR_COUNT],
    /// Current jobs
    current_jobs: [Option<ActiveMotorJob>; MOTOR_COUNT],
}

impl<'a, const MOTOR_COUNT: usize> MotorController<'a, MOTOR_COUNT> {
    pub const fn new(motor_drivers: [&'a mut dyn MotorContainer<'a>; MOTOR_COUNT]) -> Self {
        let work_queues = [const { VecDeque::new() }; MOTOR_COUNT];
        let current_jobs = [const { None }; MOTOR_COUNT];
        Self {
            motor_drivers,
            work_queues,
            current_jobs,
        }
    }

    pub fn add_job(&mut self, job: MotorJob) {
        self.work_queues[job.motor_index].push_back(job);
    }

    pub fn process_jobs(&mut self, delta_ms: u16) {
        // Process jobs
        // If a job is currently running, increment wait times to run it to completion.
        // If no job, check the queue for any work.
        for i in 0..MOTOR_COUNT {
            // Check if a job is currently running
            if let Some(active_job) = &mut self.current_jobs[i] {
                active_job.increment_time(delta_ms);
                if active_job.is_active() {
                    continue;
                } else {
                    // Job is done, stop the motor
                    let motor = self.motor_drivers[i].get_motor();
                    self.current_jobs[i] = None;

                    if self.work_queues[i].is_empty() {
                        motor.drive_motor(MotorDirection::Forward, 0);
                        continue;
                    }
                }
            }

            // Check the queue for any work
            if let Some(job) = self.work_queues[i].pop_front() {
                // Get the motor driver
                let motor = self.motor_drivers[i].get_motor();
                // Set the motor to the new job
                motor.drive_motor(job.direction, job.power);
                // Activate the job
                self.current_jobs[i] = Some(job.activate_job());
            }
        }
    }
}
