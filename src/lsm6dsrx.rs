//! https://www.st.com/resource/en/datasheet/lsm6dsrx.pdf
//! 真似してる -> https://github.com/ypc2e55orj/esp32s3_playground/blob/main/imu/main/imu.cc

use std::ops::{Deref, DerefMut};

use bitflags::bitflags;

use crate::imu::*;

/// 期待する `WHO_AM_I`
const DEFAULT_WHO_AM_I: u8 = 0x6B;

/// Gyro X: 1.103957 x + 186.606155
const DAT_X_OFS_USR: i8 = -1;

/// Gyro Y: -1.892573 x + -283.435059
const DAT_Y_OFS_USR: i8 = -46;

/// Gyro Z: -3.169824 x + 62.102707
const DAT_Z_OFS_USR: i8 = 5;

/// [mdps/LSB]
#[allow(unused)]
const ANGULAR_RATE_SENSITIVITY: f64 = 70.0;

bitflags! {
    /// 8. Register mapping
    /// Table 20. Registers addresses map
    pub struct RegisterAddress: u8 {
        const FUNC_CFG_ACCESS = 0x00;
        const PIN_CTRL = 0x02;
        const S4S_TPH_L = 0x04;
        const S4S_TPH_H = 0x05;
        const S4S_RR = 0x06;
        const FIFO_CTRL1 = 0x07;
        const FIFO_CTRL2 = 0x08;
        const FIFO_CTRL3 = 0x09;
        const FIFO_CTRL4 = 0x0A;
        const COUNTER_BDR_REG1 = 0x0B;
        const COUNTER_BDR_REG2 = 0x0C;
        const INT1_CTRL = 0x0D;
        const INT2_CTRL = 0x0E;
        const WHO_AM_I = 0x0F;
        const CTRL1_XL = 0x10;
        const CTRL2_G = 0x11;
        const CTRL3_C = 0x12;
        const CTRL4_C = 0x13;
        const CTRL5_C = 0x14;
        const CTRL6_C = 0x15;
        const CTRL7_G = 0x16;
        const CTRL8_XL = 0x17;
        const CTRL9_XL = 0x18;
        const CTRL10_C = 0x19;
        const ALL_INT_SRC = 0x1A;
        const WAKE_UP_SRC = 0x1B;
        const TAP_SRC = 0x1C;
        const D6D_SRC = 0x1D;
        const STATUS_REG = 0x1E;
        const OUT_TEMP_L = 0x20;
        const OUT_TEMP_H = 0x21;
        const OUTX_L_G = 0x22;
        const OUTX_H_G = 0x23;
        const OUTY_L_G = 0x24;
        const OUTY_H_G = 0x25;
        const OUTZ_L_G = 0x26;
        const OUTZ_H_G = 0x27;
        const OUTX_L_A = 0x28;
        const OUTX_H_A = 0x29;
        const OUTY_L_A = 0x2A;
        const OUTY_H_A = 0x2B;
        const OUTZ_L_A = 0x2C;
        const OUTZ_H_A = 0x2D;
        const EMB_FUNC_STATUS_MAINPAGE = 0x35;
        const FSM_STATUS_A_MAINPAGE = 0x36;
        const FSM_STATUS_B_MAINPAGE = 0x37;
        const MLC_STATUS_MAINPAGE = 0x38;
        const STATUS_MASTER_MAINPAGE = 0x39;
        const FIFO_STATUS1 = 0x3A;
        const FIFO_STATUS2 = 0x3B;
        const TIMESTAMP0 = 0x40;
        const TIMESTAMP1 = 0x41;
        const TIMESTAMP2 = 0x42;
        const TIMESTAMP3 = 0x43;
        const TAP_CFG0 = 0x56;
        const TAP_CFG1 = 0x57;
        const TAP_CFG2 = 0x58;
        const TAP_THS_6D = 0x59;
        const INT_DUR2 = 0x5A;
        const WAKE_UP_THS = 0x5B;
        const WAKE_UP_DUR = 0x5C;
        const FREE_FALL = 0x5D;
        const MD1_CFG = 0x5E;
        const MD2_CFG = 0x5F;
        const S4S_ST_CMD_CODE = 0x60;
        const S4S_DT_REG = 0x61;
        const I3C_BUS_AVB = 0x62;
        const INTERNAL_FREQ_FINE = 0x63;
        const INT_OIS = 0x6F;
        const CTRL1_OIS = 0x70;
        const CTRL2_OIS = 0x71;
        const CTRL3_OIS = 0x72;
        const X_OFS_USR = 0x73;
        const Y_OFS_USR = 0x74;
        const Z_OFS_USR = 0x75;
        const FIFO_DATA_OUT_TAG = 0x78;
        const FIFO_DATA_OUT_X_L = 0x79;
        const FIFO_DATA_OUT_X_H = 0x7A;
        const FIFO_DATA_OUT_Y_L = 0x7B;
        const FIFO_DATA_OUT_Y_H = 0x7C;
        const FIFO_DATA_OUT_Z_L = 0x7D;
        const FIFO_DATA_OUT_Z_H = 0x7E;
    }

    /// CTRL1_XL (0x10)
    /// Accelerometer control register 1 (r/w)
    pub struct Ctrl1Xl: u8 {
        const ODR_XL3 = 0b1000_0000;
        const ODR_XL2 = 0b0100_0000;
        const ODR_XL1 = 0b0010_0000;
        const ODR_XL0 = 0b0001_0000;
        const FS1_XL = 0b0000_1000;
        const FS0_XL = 0b0000_0100;
        const LPF2_XL_EN = 0b0000_0010;
    }

    /// CTRL2_G (0x11)
    /// Gyroscope control register 2 (r/w)
    pub struct Ctrl2G: u8 {
        const ODR_G3 = 0b1000_0000;
        const ODR_G2 = 0b0100_0000;
        const ODR_G1 = 0b0010_0000;
        const ODR_G0 = 0b0001_0000;
        const FS1_G = 0b0000_1000;
        const FS0_G = 0b0000_0100;
        const FS_125 = 0b0000_0010;
        const FS_4000 = 0b0000_0001;
    }

    /// CTRL3_C (0x12)
    /// Control register 3 (r/w)
    pub struct Ctrl3C: u8 {
        /// Reboots memory content. Default value: 0
        /// (0: normal mode; 1: reboot memory content)
        /// Note: the accelerometer must be ON. This bit is automatically cleared.
        const BOOT = 0b1000_0000;
        /// Block Data Update. Default value: 0
        /// (0: continuous update;
        /// 1: output registers are not updated until MSB and LSB have been read)
        const BDU = 0b0100_0000;
        /// Interrupt activation level. Default value: 0
        /// (0: interrupt output pins active high; 1: interrupt output pins active low
        const H_LACTIVE = 0b0010_0000;
        /// Push-pull/open-drain selection on INT1 and INT2 pins. This bit must be set to '0' when H_LACTIVE is set to '1'.
        /// Default value: 0
        /// (0: push-pull mode; 1: open-drain mode)
        const PP_OD = 0b0001_0000;
        /// SPI Serial Interface Mode selection. Default value: 0
        /// (0: 4-wire interface; 1: 3-wire interface)
        const SIM = 0b0000_1000;
        /// Register address automatically incremented during a multiple byte access with a serial interface (I²C or SPI).
        /// Default value: 1
        /// (0: disabled; 1: enabled)
        const IF_INC = 0b0000_0100;
        /// Software reset. Default value: 0
        /// (0: normal mode; 1: reset device)
        /// This bit is automatically cleared.
        const SW_RESET = 0b0000_0001;
    }

    /// CTRL4_C (0x13)
    /// Control register 4 (r/w)
    pub struct Ctrl4C: u8 {
        const SLEEP_G = 0b0100_0000;
        const INT2_ON_INT1 = 0b0010_0000;
        const DRDY_MASK = 0b0000_1000;
        const I2C_DISABLE = 0b0000_0100;
        const LPF1_SEL_G = 0b0000_0010;
    }

    /// CTRL5_C (0x14)
    /// Control register 5 (r/w)
    pub struct Ctrl5C: u8 {
        const ROUNDING1 = 0b0100_0000;
        const ROUNDING0 = 0b0010_0000;
        const ST1_G = 0b0000_1000;
        const ST0_G = 0b0000_0100;
        const ST1_XL = 0b0000_0010;
        const ST0_XL = 0b0000_0001;
    }

    /// CTRL6_C (0x15)
    /// Control register 6 (r/w)
    pub struct Ctrl6C: u8 {
        const TRIG_EN = 0b1000_0000;
        const LVL1_EN = 0b0100_0000;
        const LVL2_EN = 0b0010_0000;
        const XL_HM_MODE = 0b0001_0000;
        const USR_OFF_W = 0b0000_1000;
        const FTYPE_2 = 0b0000_0100;
        const FTYPE_1 = 0b0000_0010;
        const FTYPE_0 = 0b0000_0001;
    }

    /// CTRL7_G (0x16)
    /// Control register 7 (r/w)
    pub struct Ctrl7G: u8 {
        const G_HM_MODE = 0b1000_0000;
        const HP_EN_G = 0b0100_0000;
        const HPM1_G = 0b0010_0000;
        const HPM0_G = 0b0001_0000;
        const OIS_ON_EN = 0b0000_0100;
        const USR_OFF_ON_OUT = 0b0000_0010;
        const OIS_ON = 0b0000_0001;
    }

    /// CTRL8_XL (0x17)
    /// Control register 8 (r/w)
    pub struct Ctrl8Xl: u8 {
        const HPCF_XL_2 = 0b1000_0000;
        const HPCF_XL_1 = 0b0100_0000;
        const HPCF_XL_0 = 0b0010_0000;
        const HP_REF_MODE_XL = 0b0001_0000;
        const FASTSETTL_MODE_XL = 0b0000_1000;
        const HP_SLOPE_XL_EN = 0b0000_0100;
        const LOW_PASS_ON_6D = 0b0000_0001;
    }

    /// CTRL9_XL (0x18)
    pub struct Ctrl9Xl: u8 {
        /// DEN value stored in LSB of X-axis. Default value: 1
        /// (0: DEN not stored in X-axis LSB; 1: DEN stored in X-axis LSB)
        const DEN_X = 0b1000_0000;
        /// DEN value stored in LSB of Y-axis. Default value: 1
        /// (0: DEN not stored in Y-axis LSB; 1: DEN stored in Y-axis LSB)
        const DEN_Y = 0b1000_0000;
        /// DEN value stored in LSB of Z-axis. Default value: 1
        /// (0: DEN not stored in Z-axis LSB; 1: DEN stored in Z-axis LSB)
        const DEN_Z = 0b0010_0000;
        /// DEN stamping sensor selection. Default value: 0
        /// (0: DEN pin info stamped in the gyroscope axis selected by bits [7:5];
        /// 1: DEN pin info stamped in the accelerometer axis selected by bits [7:5])
        const DEN_XL_G = 0b0001_0000;
        /// Extends DEN functionality to accelerometer sensor. Default value: 0
        /// (0: disabled; 1: enabled)
        const DEN_XL_EN = 0b0000_1000;
        /// DEN active level configuration. Default value: 0
        /// (0: active low; 1: active high)
        const DEN_LH = 0b0000_0100;
        /// Disables MIPI I3CSM communication protocol(1)
        /// (0: SPI, I²C, MIPI I3CSM interfaces enabled (default);
        /// 1: MIPI I3CSM interface disabled)
        const I3C_DISABLE = 0b0000_0010;
    }
}

impl RegisterAddress {
    /// Returns read address
    pub fn read(&self) -> u8 {
        self.bits() | 0x80
    }
}

/// Device driver for [LSM6DSRX](https://www.st.com/ja/mems-and-sensors/lsm6dsrx.html)
pub struct Lsm6sdrx<D> {
    device: D,
}

impl<D> Deref for Lsm6sdrx<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl<D> DerefMut for Lsm6sdrx<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}

/// Implementation for SpiDevice
mod spi {
    use std::error::Error as StdError;

    use anyhow::{ensure, Context as _, Result};
    use embedded_hal::spi::{Operation, SpiDevice};

    use super::*;

    impl<D> Lsm6sdrx<D>
    where
        D: SpiDevice,
        <D as embedded_hal::spi::ErrorType>::Error: StdError + Sync + Send + 'static,
    {
        pub fn new(mut device: D) -> Result<Lsm6sdrx<D>> {
            // check device
            {
                let who_am_i = read_reg_u8(&mut device, RegisterAddress::WHO_AM_I)
                    .context("Failed to read `WHO_AM_I` register.")?;
                ensure!(who_am_i == DEFAULT_WHO_AM_I, "Incorrect device");
            }

            // reset device
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL3_C)
                    .map(Ctrl3C::from_bits_retain)
                    .context("Failed to read `CTRL3_C` register")?;
                reg.insert(Ctrl3C::SW_RESET);
                write_reg_u8(&mut device, RegisterAddress::CTRL3_C, reg.bits())
                    .context("Failed to write `CTRL3_C` register.")?;
            }

            // I3C を無効化
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL9_XL)
                    .map(Ctrl9Xl::from_bits_retain)
                    .context("Failed to read `CTRL9_XL` register")?;
                reg.insert(Ctrl9Xl::I3C_DISABLE);
                write_reg_u8(&mut device, RegisterAddress::CTRL9_XL, reg.bits())
                    .context("Failed to write `CTRL9_XL` register.")?;
            }

            // 読みだしているレジスタは更新しない
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL3_C)
                    .map(Ctrl3C::from_bits_retain)
                    .context("Failed to read `CTRL3_C` register")?;
                reg.insert(Ctrl3C::BDU);
                write_reg_u8(&mut device, RegisterAddress::CTRL3_C, reg.bits())
                    .context("Failed to write `CTRL3_C` register.")?;
            }

            // 加速度計の設定
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL1_XL)
                    .map(Ctrl1Xl::from_bits_retain)
                    .context("Failed to read `CTRL1_XL` register")?;
                // 出力レートを 1.66Khz に設定
                {
                    reg.insert(Ctrl1Xl::ODR_XL3);
                    reg.remove(Ctrl1Xl::ODR_XL2);
                    reg.remove(Ctrl1Xl::ODR_XL1);
                    reg.remove(Ctrl1Xl::ODR_XL0);
                }
                // スケールを +-2g に設定
                {
                    reg.remove(Ctrl1Xl::FS0_XL);
                    reg.remove(Ctrl1Xl::FS1_XL);
                }
                // LPF2 を有効
                reg.insert(Ctrl1Xl::LPF2_XL_EN);
                write_reg_u8(&mut device, RegisterAddress::CTRL1_XL, reg.bits())
                    .context("Failed to write `CTRL1_XL` register.")?;
            }

            // フィルタをLow pass, ODR/10に設定
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL8_XL)
                    .map(Ctrl8Xl::from_bits_retain)
                    .context("Failed to read `CTRL8_XL` register")?;
                reg.insert(Ctrl8Xl::HPCF_XL_0);
                reg.remove(Ctrl8Xl::HPCF_XL_1);
                reg.remove(Ctrl8Xl::HPCF_XL_2);
                write_reg_u8(&mut device, RegisterAddress::CTRL8_XL, reg.bits())
                    .context("Failed to write `CTRL8_XL` register.")?;
            }

            // オフセットの重みを2^-10 g/LSBに設定
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL6_C)
                    .map(Ctrl6C::from_bits_retain)
                    .context("Failed to read `CTRL6_C` register")?;
                reg.remove(Ctrl6C::USR_OFF_W);
                write_reg_u8(&mut device, RegisterAddress::CTRL6_C, reg.bits())
                    .context("Failed to write `CTRL6_C` register.")?;
            }

            // オフセットを有効
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL7_G)
                    .map(Ctrl7G::from_bits_retain)
                    .context("Failed to read `CTRL7_G` register")?;
                reg.insert(Ctrl7G::USR_OFF_ON_OUT);
                write_reg_u8(&mut device, RegisterAddress::CTRL7_G, reg.bits())
                    .context("Failed to write `CTRL7_G` register.")?;
                write_reg_u8(&mut device, RegisterAddress::X_OFS_USR, DAT_X_OFS_USR as u8)
                    .context("Failed to write `X_OFS_USR` register.")?;
                write_reg_u8(&mut device, RegisterAddress::Y_OFS_USR, DAT_Y_OFS_USR as u8)
                    .context("Failed to write `Y_OFS_USR` register.")?;
                write_reg_u8(&mut device, RegisterAddress::Z_OFS_USR, DAT_Z_OFS_USR as u8)
                    .context("Failed to write `Z_OFS_USR` register.")?;
            }

            // 角速度計の設定
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL2_G)
                    .map(Ctrl2G::from_bits_retain)
                    .context("Failed to read `CTRL2_G` register")?;
                // 出力レートを 1.66Khz に設定
                {
                    reg.insert(Ctrl2G::ODR_G3);
                    reg.remove(Ctrl2G::ODR_G2);
                    reg.remove(Ctrl2G::ODR_G1);
                    reg.remove(Ctrl2G::ODR_G0);
                }
                // スケールを +-2000dps に設定
                {
                    reg.insert(Ctrl2G::FS1_G);
                    reg.insert(Ctrl2G::FS0_G);
                    reg.remove(Ctrl2G::FS_125);
                    reg.remove(Ctrl2G::FS_4000);
                }
                write_reg_u8(&mut device, RegisterAddress::CTRL2_G, reg.bits())
                    .context("Failed to write `CTRL2_G` register.")?;
            }

            // LPF1 を有効
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL4_C)
                    .map(Ctrl4C::from_bits_retain)
                    .context("Failed to read `CTRL4_C` register")?;
                reg.insert(Ctrl4C::LPF1_SEL_G);
                write_reg_u8(&mut device, RegisterAddress::CTRL4_C, reg.bits())
                    .context("Failed to write `CTRL4_C` register.")?;
            }

            // これは何
            {
                let mut reg = read_reg_u8(&mut device, RegisterAddress::CTRL6_C)
                    .map(Ctrl6C::from_bits_retain)
                    .context("Failed to read `CTRL6_C` register")?;
                reg.remove(Ctrl6C::FTYPE_2);
                reg.insert(Ctrl6C::FTYPE_1);
                reg.remove(Ctrl6C::FTYPE_0);
                write_reg_u8(&mut device, RegisterAddress::CTRL6_C, reg.bits())
                    .context("Failed to write `CTRL6_C` register.")?;
            }

            Ok(Lsm6sdrx { device })
        }

        /// 加速度を取得する
        pub fn fetch_acceleration(&mut self) -> Result<Acceleration> {
            /// [mg/LSB]
            const LINEAR_ACCELERATION_SENSITIVITY: f64 = 0.061;

            #[repr(packed)]
            #[derive(Default, Debug)]
            struct RxBuffer {
                pub x: i16,
                pub y: i16,
                pub z: i16,
            }

            let mut buffer = [u8::MIN; std::mem::size_of::<RxBuffer>()];
            self.device
                .transaction(&mut [
                    Operation::Write(&[RegisterAddress::OUTX_L_G.read()]),
                    Operation::Read(&mut buffer),
                ])
                .context("Failed to run transaction")?;

            let buffer = unsafe { &*(buffer.as_ptr() as *const RxBuffer) };

            log::info!("buffer = {buffer:?}");

            let acceleration = Acceleration::new(
                (buffer.x as f64) * LINEAR_ACCELERATION_SENSITIVITY,
                (buffer.y as f64) * LINEAR_ACCELERATION_SENSITIVITY,
                (buffer.z as f64) * LINEAR_ACCELERATION_SENSITIVITY,
            );

            Ok(acceleration)
        }
    }

    impl<D> Accelerometer for Lsm6sdrx<D>
    where
        D: SpiDevice,
        <D as embedded_hal::spi::ErrorType>::Error: StdError + Sync + Send + 'static,
    {
        fn fetch(&mut self) -> Result<Acceleration> {
            self.fetch_acceleration()
        }
    }

    #[inline]
    fn read_reg_u8<D: SpiDevice>(device: &mut D, addr: RegisterAddress) -> Result<u8, D::Error> {
        let write_buf = [addr.read()];
        let mut read_buf = [u8::MIN];
        device.transaction(&mut [Operation::Write(&write_buf), Operation::Read(&mut read_buf)])?;
        Ok(read_buf[0])
    }

    #[inline]
    fn write_reg_u8<D: SpiDevice>(
        device: &mut D,
        addr: RegisterAddress,
        data: u8,
    ) -> Result<(), D::Error> {
        device.transaction(&mut [Operation::Write(&[addr.bits()]), Operation::Write(&[data])])?;
        Ok(())
    }
}
