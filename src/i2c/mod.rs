extern crate byteorder;
extern crate cortex_m_semihosting;
extern crate embedded_hal as hal;

use core::marker::PhantomData;
use cortex_m_semihosting::hprintln;
use embedded_hal::blocking::i2c::{Write, WriteRead};

const I2C_ADDRESS_PRIMARY: u8 = 0x68;
//const I2C_ADDRESS_SECONDARY: u8 = 0x69;
const BMI160_CHIP_ID: u8 = 0xD1;

const ERR_REG_MASK: u8 = 0x0F;

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
  Error = 0x02,
  Command = 0x7E,
  GyroData = 0x0C,
  // AccelData = 0x12,
  AccelConfig = 0x40,
  AccelRange = 0x41,
  GyroConfig = 0x42,
  GyroRange = 0x43,
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
  pub config: super::config::BMI160Config,
}

impl<I2C, Delay, E> BMI160<I2C, Delay>
where
  I2C: WriteRead<Error = E> + Write<Error = E>,
  Delay: embedded_hal::blocking::delay::DelayMs<u8>,
{
  /// Creates a new driver from a I2C peripheral
  /// Reads the chip id from register 0x00
  pub fn new(
    i2c: &mut I2C,
    delay: &mut Delay,
    config: super::config::BMI160Config,
  ) -> Result<Self, E> {
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
      config: config,
    };
    // Perform a soft reset of the device
    hprintln!("BMI160: Soft reset").unwrap();
    i2c.write(
      sensor.addr,
      &[Register::Command.addr(), Command::SoftReset.val()],
    )?;
    delay.delay_ms(2);
    sensor.configure(i2c, delay)?;
    Ok(sensor)
  }

  pub fn configure(&self, i2c: &mut I2C, delay: &mut Delay) -> Result<(), E> {
    hprintln!("BMI160: Configuring").unwrap();
    // Set accel configuration
    self.config_accelerometer(i2c)?;
    // Set gyro configuration
    self.config_gyroscope(i2c)?;
    // Set accel power mode
    self.set_accel_power_mode(i2c, delay)?;
    // Set gyro power mode
    self.set_gyro_power_mode(i2c, delay)?;
    // Check for invalid setting combinations
    self.check_config(i2c)?;
    Ok(())
  }

  fn check_config(&self, i2c: &mut I2C) -> Result<(), E> {
    let mut buf: [u8; 1] = [0; 1];
    i2c.write_read(self.addr, &[Register::Error.addr()], &mut buf)?;
    let data = (buf[0] >> 1) & ERR_REG_MASK;
    if data == 1 {
      hprintln!("BMI160: Accel ODR + BW Invalid").unwrap();
    } else if data == 2 {
      hprintln!("BMI160: Gyro ODR + BW Invalid").unwrap();
    } else if data == 3 {
      hprintln!("BMI160: Prefilter Interrupt Invalid").unwrap();
    } else if data == 7 {
      hprintln!("BMI160: Prefilter Invalid").unwrap();
    }
    Ok(())
  }

  fn set_gyro_power_mode(&self, i2c: &mut I2C, delay: &mut Delay) -> Result<(), E> {
    // apply power mode
    i2c.write(
      self.addr,
      &[
        Register::Command.addr(),
        self.config.gyro_config.power as u8,
      ],
    )?;
    // If exiting suspend, need a 80ms delay - see data sheet table 24
    // @TOD Store previous value to compare to
    // if prev_config.gyro_config.power == super::config::GyroPowerMode::SuspendMode {
    delay.delay_ms(80);
    // }
    // If transitioning for fast startup to normal mode, delay is only 10 ms instead of 80
    // else if (prev_config.gyro_config.power == super::config::GyroPowerMode::FastStartupMode) &&
    //        (self.config.gyro_config.power == super::config::GyroPowerMode::NormalMode) {
    // delay.delay_ms(10);
    // }
    Ok(())
  }

  fn set_accel_power_mode(&self, i2c: &mut I2C, delay: &mut Delay) -> Result<(), E> {
    // apply power mode
    i2c.write(
      self.addr,
      &[
        Register::Command.addr(),
        self.config.accel_config.power as u8,
      ],
    )?;
    // If transitioning from suspend, need a 3.8ms delay before processing any other commands
    // See data sheet table 24
    // @TODO Store previous value to compare to
    // if prev_config.accel_config.power == super::config::AccelPowerMode::SuspendMode {
    // delay.delay_ms(4);
    delay.delay_ms(40);
    //}
    Ok(())
  }

  fn config_accelerometer(&self, i2c: &mut I2C) -> Result<(), E> {
    let mut buf: [u8; 2] = [0; 2];
    i2c.write_read(self.addr, &[Register::AccelConfig.addr()], &mut buf)?;
    let updated = self.config.apply_accel_config(buf);
    // Write registers back to the device
    i2c.write(self.addr, &[Register::AccelConfig.addr(), updated[0]])?;
    i2c.write(self.addr, &[Register::AccelRange.addr(), updated[1]])?;

    Ok(())
  }

  fn config_gyroscope(&self, i2c: &mut I2C) -> Result<(), E> {
    let mut buf: [u8; 2] = [0; 2];
    i2c.write_read(self.addr, &[Register::GyroConfig.addr()], &mut buf)?;
    let updated = self.config.apply_gyro_config(buf);
    // Write registers back to the device
    i2c.write(self.addr, &[Register::GyroConfig.addr(), updated[0]])?;
    i2c.write(self.addr, &[Register::GyroRange.addr(), updated[1]])?;

    Ok(())
  }
  pub fn get_sensor_data(&self, i2c: &mut I2C) -> Result<BMI160Reading, E> {
    // Starting at GyroDataAddress, read 15 registers. First 6 are gyro data in msb/lsb pairs.
    // Second 6 are accel data also in msb/lsb pairs. Last 3 are timing data.
    let mut buf: [u8; 15] = [0; 15];

    i2c.write_read(self.addr, &[Register::GyroData.addr()], &mut buf)?;
    let gx_lsb = buf[0] as u16;
    let gx_msb = buf[1] as u16;
    let gx: i16 = ((gx_msb << 8) | gx_lsb) as i16;

    let gy_lsb = buf[2] as u16;
    let gy_msb = buf[3] as u16;
    let gy: i16 = ((gy_msb << 8) | gy_lsb) as i16;
    let gz_lsb = buf[4] as u16;
    let gz_msb = buf[5] as u16;
    let gz: i16 = ((gz_msb << 8) | gz_lsb) as i16;

    let ax_lsb = buf[6] as u16;
    let ax_msb = buf[7] as u16;
    let ax: i16 = ((ax_msb << 8) | ax_lsb) as i16;

    let ay_lsb = buf[8] as u16;
    let ay_msb = buf[9] as u16;
    let ay: i16 = ((ay_msb << 8) | ay_lsb) as i16;

    let az_lsb = buf[10] as u16;
    let az_msb = buf[11] as u16;
    let az: i16 = ((az_msb << 8) | az_lsb) as i16;
    let time: u32 = 0u32; // LittleEndian::read_u32(&buf[12..14]);

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
