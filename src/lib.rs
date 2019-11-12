//! # Bosch BMI160 Accelerometer and Gyroscope Library
//!
//! The BMI160 sensor utilizes both the I2C and SPI interfaces.
//! This crate utilizes the embedded_hal constructs to provide a device
//! neutral implementation.
//!
//! See the main datasheet for this sensor [Data Sheet](https://ae-bst.resource.bosch.com/media/_tech/media/datasheets/BST-BMI160-DS000.pdf)
//!

#![no_std]

pub mod config;
pub mod i2c;
