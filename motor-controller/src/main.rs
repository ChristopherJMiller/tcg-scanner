pub mod controller;
pub mod motors;
pub mod wifi;

use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    thread::sleep,
    time::Duration,
};

use anyhow::{anyhow, Result};
use controller::{job::MotorJob, motor::Motor, MotorController};
use embedded_svc::http::Headers;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::PinDriver,
        ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
        prelude::*,
    },
    http::{
        server::{Configuration, EspHttpServer},
        Method,
    },
    io::{EspIOError, Read, Write},
};
use log::info;
use motors::TcgMotor;
use wifi::wifi;

// Max payload length
const MAX_LEN: usize = 128;

pub struct Config {
    wifi_ssid: &'static str,
    wifi_psk: &'static str,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting up...");

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = Config {
        wifi_ssid: option_env!("CONFIG_WIFI_SSID").ok_or_else(|| anyhow!("Missing SSID"))?,
        wifi_psk: option_env!("CONFIG_WIFI_PSK").ok_or_else(|| anyhow!("Missing PSK"))?,
    };

    // Connect to the Wi-Fi network
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    // Configure the motors
    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::default().frequency(1.kHz().into()),
    )?;

    let mut top_drawer = TcgMotor::TopDrawer0(Motor::new(
        PinDriver::output(peripherals.pins.gpio19)?,
        PinDriver::output(peripherals.pins.gpio18)?,
        LedcDriver::new(
            peripherals.ledc.channel0,
            timer_driver,
            peripherals.pins.gpio22,
        )?,
    ));

    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer1,
        &TimerConfig::default().frequency(1.kHz().into()),
    )?;

    let mut lower_back = TcgMotor::BackLower1(Motor::new(
        PinDriver::output(peripherals.pins.gpio5)?,
        PinDriver::output(peripherals.pins.gpio17)?,
        LedcDriver::new(
            peripherals.ledc.channel1,
            timer_driver,
            peripherals.pins.gpio1,
        )?,
    ));

    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer2,
        &TimerConfig::default().frequency(1.kHz().into()),
    )?;

    let mut lower_front = TcgMotor::FrontLower2(Motor::new(
        PinDriver::output(peripherals.pins.gpio16)?,
        PinDriver::output(peripherals.pins.gpio4)?,
        LedcDriver::new(
            peripherals.ledc.channel2,
            timer_driver,
            peripherals.pins.gpio3,
        )?,
    ));

    let mut motor_controller =
        MotorController::new([&mut top_drawer, &mut lower_back, &mut lower_front]);

    // Create a queue to hold the jobs
    // Instead of directly queuing onto the motor controller, allows that to be controlled by the main thread
    // and instead let interrupts queue here
    let to_queue = Arc::new(RwLock::new(VecDeque::new()));
    let to_queue_clone = to_queue.clone();

    // Set the HTTP server
    let mut server = EspHttpServer::new(&Configuration::default())?;

    // Queue job Handler
    server.fn_handler(
        "/job",
        Method::Post,
        move |mut req: esp_idf_svc::http::server::Request<
            &mut esp_idf_svc::http::server::EspHttpConnection<'_>,
        >|
              -> core::result::Result<(), EspIOError> {
            let len = req.content_len().unwrap_or(0) as usize;

            if len > MAX_LEN {
                req.into_status_response(413)?
                    .write_all("Request too big".as_bytes())?;
                return Ok(());
            }

            let mut buf = vec![0u8; len];
            if let Err(err) = req.read_exact(&mut buf) {
                req.into_status_response(400)?
                    .write_all(format!("Failed to read request: {}", err).as_bytes())?;
                return Ok(());
            }

            let job = serde_json::from_slice::<MotorJob>(&buf);
            if let Err(err) = job {
                req.into_status_response(400)?
                    .write_all(format!("Failed to parse request: {}", err).as_bytes())?;
                return Ok(());
            }

            let job = job.unwrap();
            if job.motor_index > 2 {
                req.into_status_response(400)?
                    .write_all(b"Invalid motor index")?;
                return Ok(());
            }
            if job.power > 100 {
                req.into_status_response(400)?
                    .write_all(b"Invalid power level")?;
                return Ok(());
            }

            info!("Received job: {:?}", job);

            {
                let queue_arc = to_queue.clone();
                let to_queue = queue_arc.write();
                if let Err(err) = to_queue {
                    req.into_status_response(500)?
                        .write_all(format!("Failed to acquire lock: {}", err).as_bytes())?;
                    return Ok(());
                }
                to_queue.unwrap().push_back(job);
            }

            let mut response = req.into_ok_response()?;
            response.write_all(b"OK")?;
            Ok(())
        },
    )?;

    println!("Server awaiting connection");

    loop {
        // Check if there are any jobs in the queue
        let queue_arc = to_queue_clone.clone();
        let to_queue = queue_arc.read();
        if let Err(err) = to_queue {
            println!("Failed to acquire lock: {}", err);
            continue;
        }
        let to_queue = to_queue.unwrap();
        if !to_queue.is_empty() {
            drop(to_queue); // Drop the read lock to allow writing
            let to_queue = queue_arc.write();
            if let Err(err) = to_queue {
                println!("Failed to acquire lock: {}", err);
                continue;
            }
            let mut to_queue = to_queue.unwrap();
            // Process the jobs
            while let Some(job) = to_queue.pop_front() {
                // Add the job to the motor controller
                motor_controller.add_job(job);
            }
        }
        motor_controller.process_jobs(100);
        sleep(Duration::from_millis(100));
    }
}
