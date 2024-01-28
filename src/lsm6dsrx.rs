//! https://www.st.com/resource/en/datasheet/lsm6dsrx.pdf

use std::ops::{Deref, DerefMut};

use bitflags::bitflags;

use crate::imu::*;

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

    /// CTRL3_C (0x12)
    pub struct Control3Flags: u8 {
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
}

const DEFAULT_WHO_AM_I: u8 = 0x6B;

pub struct Lsm6sdrx<D> {
    #[allow(unused)]
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

mod spi {
    use std::error::Error as StdError;

    use anyhow::{bail, ensure, Context as _, Result};
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
                    .map(Control3Flags::from_bits_retain)
                    .context("Failed to read `REG_CTRL3_C` register")?;
                reg.insert(Control3Flags::SW_RESET);
                write_reg_u8(&mut device, RegisterAddress::CTRL3_C, reg.bits())
                    .context("Failed to write `REG_CTRL3_C` register.")?;
            }

            // TODO: なんかいろいろしないといけない

            Ok(Lsm6sdrx { device })
        }
    }

    impl<D> Imu for Lsm6sdrx<D>
    where
        D: SpiDevice,
        <D as embedded_hal::spi::ErrorType>::Error: StdError + Sync + Send + 'static,
    {
        fn fetch_acceleration(&mut self) -> Result<Acceleration> {
            bail!("TODO")
        }
    }

    #[inline]
    fn read_reg_u8<D: SpiDevice>(device: &mut D, addr: RegisterAddress) -> Result<u8, D::Error> {
        let write_buf = [addr.bits() | 0x80];
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
