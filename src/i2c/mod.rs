extern crate cortex_m_semihosting;
extern crate embedded_hal as hal;

use core::marker::PhantomData;
use cortex_m_semihosting::hprintln;
use embedded_hal::blocking::i2c::{Write, WriteRead};

const I2C_ADDRESS_PRIMARY: u8 = 0x68;
const I2C_ADDRESS_SECONDARY: u8 = 0x69;
const BMI160_CHIP_ID: u8 = 0xD1;

#[derive(Copy, Clone)]
enum Register {
  ChipId = 0x00,
}
impl Register {
  pub fn addr(&self) -> u8 {
    *self as u8
  }
}
pub struct BMI160<I2C, Delay> {
  pub i2c: PhantomData<I2C>,
  pub delay: PhantomData<Delay>,
  pub addr: u8,
}

impl<I2C, Delay, E> BMI160<I2C, Delay>
where
  I2C: WriteRead<Error = E> + Write<Error = E>,
  Delay: embedded_hal::blocking::delay::DelayMs<u8>,
{
  /// Creates a new driver from a I2C peripheral
  /// Reads the chip id from register 0x00
  pub fn new(i2c: &mut I2C) -> Result<Self, E> {
    let mut buf: [u8; 1] = [0; 1];
    hprintln!("BMI160: Reading chip id").unwrap();
    i2c.write_read(I2C_ADDRESS_PRIMARY, &[Register::ChipId.addr()], &mut buf)?;
    hprintln!(
      "BMI160: Read Chip ID -> {:?} <> Expected {}",
      buf,
      BMI160_CHIP_ID
    )
    .unwrap();
    let sensor = BMI160 {
      i2c: PhantomData,
      delay: PhantomData,
      addr: I2C_ADDRESS_SECONDARY,
    };
    Ok(sensor)
  }
}
