use anyhow::{ensure, Context, Result};
use embedded_hal::spi::MODE_3;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::Pins,
    interrupt::IntrFlags,
    peripherals::Peripherals,
    spi::{self, Dma, SpiDeviceDriver, SpiDriver, SpiDriverConfig, SPI2},
    units::MegaHertz,
};

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let Peripherals {
        spi2,
        pins:
            Pins {
                gpio5: sdi,    // MISO/SDI
                gpio6: sdo,    // MOSI/SDO
                gpio7: sclk,   // sclk
                gpio8: spi_cs, // SPI CS
                ..
            },
        ..
    } = Peripherals::take().context("Failed to take peripherals.")?;

    let driver_cfg = SpiDriverConfig::new()
        .dma(Dma::Auto(16))
        .intr_flags(IntrFlags::Iram.into());
    let driver = SpiDriver::new::<SPI2>(spi2, sclk, sdo, Some(sdi), &driver_cfg)
        .context("Failed to create SPI driver.")?;

    let device_cfg = spi::config::Config::new().baudrate(MegaHertz::from(10).into()).data_mode(MODE_3);
    let mut device = SpiDeviceDriver::new(driver, Some(spi_cs), &device_cfg)
        .context("Failed to create SPI device.")?;

    loop {
        FreeRtos::delay_ms(1000);

        let mut read_buf = [0u8; 1];
        let reg = 0x0F_u8 | 0x80;
        device
            .transfer(&mut read_buf, &[reg])
            .context("Failed to transfer WHO_AM_I")?;
        ensure!(read_buf == [0x6B]);
        // log::info!("buf = {read_buf:?}");
    }
}
