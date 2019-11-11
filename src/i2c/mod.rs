extern crate byteorder;
extern crate cortex_m_semihosting;
extern crate embedded_hal as hal;

use byteorder::{BigEndian, ByteOrder};
use core::marker::PhantomData;
use cortex_m_semihosting::hprintln;
use embedded_hal::blocking::i2c::{Write, WriteRead};

const I2C_ADDRESS_PRIMARY: u8 = 0x68;
const I2C_ADDRESS_SECONDARY: u8 = 0x69;
const BMI160_CHIP_ID: u8 = 0xD1;

pub struct BMI160Reading {
  pub accel_x: i16,
  pub accel_y: i16,
  pub accel_z: i16,
  pub gyro_x: i16,
  pub gyro_y: i16,
  pub gyro_z: i16,
  pub time: u32,
}

#[derive(Copy, Clone)]
enum Command {
  SoftReset = 0xb6,
}

#[derive(Copy, Clone)]
enum Register {
  ChipId = 0x00,
  Command = 0x7E,
  GyroData = 0x0C,
  AccelData = 0x12,
}
impl Register {
  pub fn addr(&self) -> u8 {
    *self as u8
  }
}
impl Command {
  pub fn val(&self) -> u8 {
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
  pub fn new(i2c: &mut I2C, delay: &mut Delay) -> Result<Self, E> {
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
      addr: I2C_ADDRESS_PRIMARY,
    };
    // Perform a soft reset of the device
    i2c.write(
      sensor.addr,
      &[Register::Command.addr(), Command::SoftReset.val()],
    )?;
    delay.delay_ms(2);
    Ok(sensor)
  }

  pub fn get_sensor_data(&self, i2c: &mut I2C) -> Result<BMI160Reading, E> {
    // Starting at GyroDataAddress, read 15 registers. First 6 are gyro data in msb/lsb pairs.
    // Second 6 are accel data also in msb/lsb pairs. Last 3 are timing data.
    let mut buf: [u8; 15] = [0; 15];

    i2c.write_read(self.addr, &[Register::GyroData.addr()], &mut buf)?;
    let gx: i16 = ((buf[1] << 8) | buf[0]) as i16;
    let gy: i16 = ((buf[3] << 8) | buf[2]) as i16;
    let gz: i16 = ((buf[5] << 8) | buf[4]) as i16;
    let ax: i16 = ((buf[7] << 8) | buf[6]) as i16;
    let ay: i16 = ((buf[9] << 8) | buf[8]) as i16;
    let az: i16 = ((buf[11] << 8) | buf[10]) as i16;
    let time: u32 = ((buf[14] << 16) | (buf[13] << 8) | buf[12]) as u32;

    Ok(BMI160Reading {
      accel_x: ax,
      accel_y: ay,
      accel_z: az,
      gyro_x: gx,
      gyro_y: gy,
      gyro_z: gz,
      time: time,
    })
  }
}
