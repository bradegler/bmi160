//! Configuration settings for the BMI160

// Mask definitions
const ACCEL_BW_MASK: u8 = 0x70;
const ACCEL_ODR_MASK: u8 = 0x0F;
const ACCEL_UNDERSAMPLING_MASK: u8 = 0x80;
const ACCEL_RANGE_MASK: u8 = 0x0F;
const GYRO_BW_MASK: u8 = 0x30;
const GYRO_ODR_MASK: u8 = 0x0F;
const GYRO_RANGE_MASK: u8 = 0x07;

#[derive(Copy, Clone, PartialEq)]
pub enum AccelPowerMode {
  SuspendMode = 0x10,
  NormalMode = 0x11,
  LowPowerMode = 0x12,
}

#[derive(Copy, Clone)]
pub enum AccelOutputDataRate {
  OdrReserved = 0x00,
  Odr0_78Hz = 0x01,
  Odr1_56Hz = 0x02,
  Odr3_12Hz = 0x03,
  Odr6_25Hz = 0x04,
  Odr12_5Hz = 0x05,
  Odr25Hz = 0x06,
  Odr50Hz = 0x07,
  Odr100Hz = 0x08,
  Odr200Hz = 0x09,
  Odr400Hz = 0x0A,
  Odr800Hz = 0x0B,
  Odr1600Hz = 0x0C,
  OdrReserved0 = 0x0D,
  OdrReserved1 = 0x0E,
  OdrReserved2 = 0x0F,
}

#[derive(Copy, Clone)]
pub enum AccelRange {
  Range2G = 0x03,
  Range4G = 0x05,
  Range8G = 0x08,
  Range16G = 0x0C,
}

#[derive(Copy, Clone)]
pub enum AccelBandwidth {
  BwOsr4Avg1 = 0x00,
  BwOsr2Avg2 = 0x01,
  BwNormalAvg4 = 0x02,
  BwResAvg8 = 0x03,
  BwResAvg16 = 0x04,
  BwResAvg32 = 0x05,
  BwResAvg64 = 0x06,
  BwResAvg128 = 0x07,
}

/// Configuration data for the accelerometer
#[derive(Copy, Clone)]
pub struct AccelConfig {
  // power mode
  pub power: AccelPowerMode,
  // output data rate
  pub odr: AccelOutputDataRate,
  // range
  pub range: AccelRange,
  // bandwidth
  pub bandwidth: AccelBandwidth,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GyroPowerMode {
  SuspendMode = 0x14,
  NormalMode = 0x15,
  FastStartupMode = 0x17,
}

#[derive(Copy, Clone)]
pub enum GyroOutputDataRate {
  OdrReserved = 0x00,
  Odr25Hz = 0x06,
  Odr50Hz = 0x07,
  Odr100Hz = 0x08,
  Odr200Hz = 0x09,
  Odr400Hz = 0x0A,
  Odr800Hz = 0x0B,
  Odr1600Hz = 0x0C,
  Odr3200Hz = 0x0D,
}

#[derive(Copy, Clone)]
pub enum GyroRange {
  Range2000Dps = 0x00,
  Range1000Dps = 0x01,
  Range500Dps = 0x02,
  Range250Dps = 0x03,
  Range125Dps = 0x04,
}

#[derive(Copy, Clone)]
pub enum GyroBandwidth {
  BwOsr4Mode = 0x00,
  BwOsr2MOde = 0x01,
  BwNormalMode = 0x02,
}

/// Configuration data for the gyroscope
#[derive(Copy, Clone)]
pub struct GyroConfig {
  // power mode
  pub power: GyroPowerMode,
  // output data rate
  pub odr: GyroOutputDataRate,
  // range
  pub range: GyroRange,
  // bandwidth
  pub bandwidth: GyroBandwidth,
}

#[derive(Copy, Clone)]
pub struct BMI160Config {
  pub accel_config: AccelConfig,
  pub gyro_config: GyroConfig,
}

pub fn default() -> BMI160Config {
  BMI160Config {
    accel_config: AccelConfig {
      power: AccelPowerMode::SuspendMode,
      odr: AccelOutputDataRate::Odr100Hz,
      range: AccelRange::Range2G,
      bandwidth: AccelBandwidth::BwNormalAvg4,
    },
    gyro_config: GyroConfig {
      power: GyroPowerMode::SuspendMode,
      odr: GyroOutputDataRate::Odr100Hz,
      range: GyroRange::Range2000Dps,
      bandwidth: GyroBandwidth::BwNormalMode,
    },
  }
}

impl BMI160Config {
  pub fn apply_accel_config(&self, buf: [u8; 2]) -> [u8; 2] {
    let seg1 = buf[0];
    // Process ODR
    let odr = self.accel_config.odr as u8;
    let temp = seg1 & !ACCEL_ODR_MASK;
    let seg1 = temp | (odr & ACCEL_ODR_MASK);
    // Process BW
    let bw = self.accel_config.bandwidth as u8;
    let temp = seg1 & !ACCEL_BW_MASK;
    let seg1 = temp | ((bw << 4) & ACCEL_BW_MASK);
    // Process undersampling
    let seg1 = if self.accel_config.power == AccelPowerMode::LowPowerMode {
      let temp = seg1 & !ACCEL_UNDERSAMPLING_MASK;
      temp | ((1 << 7) & ACCEL_UNDERSAMPLING_MASK)
    // @TODO - In this case the pre filter needs disabled as well
    // Basically 2 zero byte write at adddress BMI160_INT_DATA_0_ADDR
    } else {
      if seg1 & ACCEL_UNDERSAMPLING_MASK == 1 {
        // Remove the undersampling if it is already configured
        seg1 & !ACCEL_UNDERSAMPLING_MASK
      } else {
        seg1
      }
    };
    // Process Range
    let range = self.accel_config.range as u8;
    let seg2 = buf[1];
    let temp = seg2 & !ACCEL_RANGE_MASK;
    let seg2 = temp | (range & ACCEL_RANGE_MASK);
    [seg1, seg2]
  }
  pub fn apply_gyro_config(&self, buf: [u8; 2]) -> [u8; 2] {
    let seg1 = buf[0];
    // Process ODR
    let odr = self.gyro_config.odr as u8;
    let temp = seg1 & !GYRO_ODR_MASK;
    let seg1 = temp | (odr & GYRO_ODR_MASK);
    // Process BW
    let bw = self.gyro_config.bandwidth as u8;
    let temp = seg1 & !GYRO_BW_MASK;
    let seg1 = temp | ((bw << 4) & GYRO_BW_MASK);
    // Process Range
    let range = self.gyro_config.range as u8;
    let seg2 = buf[1];
    let temp = seg2 & !GYRO_RANGE_MASK;
    let seg2 = temp | (range & GYRO_RANGE_MASK);
    [seg1, seg2]
  }
}
