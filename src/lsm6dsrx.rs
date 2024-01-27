//! https://www.st.com/resource/en/datasheet/lsm6dsrx.pdf

use std::ops::{Deref, DerefMut};

use bitflags::bitflags;

use crate::imu::*;

mod reg {
    pub(super) mod addr {
        pub const WHO_AM_I: u8 = 0x0F;
        pub const REG_CTRL3_C: u8 = 0x12;
    }
    pub(super) mod def {
        pub const WHO_AM_I: u8 = 0x6B;
    }
}

bitflags! {
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
                let who_am_i = read_reg_u8(&mut device, reg::addr::WHO_AM_I)
                    .context("Failed to read `WHO_AM_I` register.")?;
                ensure!(who_am_i == reg::def::WHO_AM_I, "Incorrect device");
            }

            // reset device
            {
                let mut reg = read_reg_u8(&mut device, reg::addr::REG_CTRL3_C)
                    .map(Control3Flags::from_bits_retain)
                    .context("Failed to read `REG_CTRL3_C` register")?;
                reg.insert(Control3Flags::SW_RESET);
                write_reg_u8(&mut device, reg::addr::REG_CTRL3_C, reg.bits())
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
    fn read_reg_u8<D: SpiDevice>(device: &mut D, addr: u8) -> Result<u8, D::Error> {
        let write_buf = [addr | 0x80];
        let mut read_buf = [u8::MIN];
        device.transaction(&mut [Operation::Write(&write_buf), Operation::Read(&mut read_buf)])?;
        Ok(read_buf[0])
    }

    #[inline]
    fn write_reg_u8<D: SpiDevice>(device: &mut D, addr: u8, data: u8) -> Result<(), D::Error> {
        device.transaction(&mut [Operation::Write(&[addr]), Operation::Write(&[data])])?;
        Ok(())
    }
}
