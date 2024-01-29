mod imu;
mod lsm6dsrx;

use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use embedded_hal::spi::MODE_3;
use esp_idf_hal::{
    gpio::Pins,
    interrupt::IntrFlags,
    modem::WifiModemPeripheral,
    peripheral::Peripheral,
    peripherals::Peripherals,
    spi::{self, Dma, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
    units::MegaHertz,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::{server::EspHttpServer, Method},
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, EspWifi},
};

use crate::{imu::Accelerometer as _, lsm6dsrx::Lsm6sdrx};

const STACK_SIZE: usize = 10240;
const WIFI_SSID: Option<&str> = option_env!("WIFI_SSID");
const WIFI_PASSWORD: Option<&str> = option_env!("WIFI_PASSWORD");

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
        modem,
        ..
    } = Peripherals::take().context("Failed to take peripherals.")?;
    let sys_loop = EspSystemEventLoop::take().context("Failed to take system event loop.")?;
    let nvs = EspDefaultNvsPartition::take().context("Failed to take nvs.")?;

    let wifi = connect_wifi(
        WIFI_SSID.expect("`WIFI_SSID` not set."),
        WIFI_PASSWORD.expect("`WIFI_PASS` not set."),
        modem,
        sys_loop,
        Some(nvs),
    )
    .context("Failed to connect wifi.")?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    log::info!("IPv4 addr: {}", ip_info.ip);

    let spi_device = {
        let driver_cfg = SpiDriverConfig::new()
            .dma(Dma::Auto(16))
            .intr_flags(IntrFlags::Iram.into());
        let driver = SpiDriver::new(spi2, sclk, sdo, Some(sdi), &driver_cfg)
            .context("Failed to create SPI driver.")?;

        let device_cfg = spi::config::Config::new()
            .baudrate(MegaHertz::from(10).into())
            .data_mode(MODE_3);

        SpiDeviceDriver::new(driver, Some(spi_cs), &device_cfg)
            .context("Failed to create SPI device.")?
    };

    let imu = Arc::new(Mutex::new(Lsm6sdrx::new(spi_device)?));
    log::info!("Lsm6sdrx initialized.");

    let mut server: EspHttpServer<'_> = create_server().context("Failed to create server.")?;

    {
        let imu = Arc::clone(&imu);
        server.fn_handler("/accel", Method::Get, move |req| -> Result<()> {
            use esp_idf_hal::io::Write;
            let mut res = req.into_ok_response()?;
            let mut imu = imu.lock().expect("Failed to lock mutex.");
            let data = imu.fetch()?;
            let json_text = serde_json::to_string_pretty(&data)?;
            writeln!(&mut res, "{json_text}")?;
            Ok(())
        })?;
    }

    {
        server.fn_handler("/gyro", Method::Get, move |req| -> Result<()> {
            use esp_idf_hal::io::Write;
            let mut res = req.into_ok_response()?;
            // そのうちジャイロもやる
            let data = None as Option<()>;
            let json_text = serde_json::to_string_pretty(&data)?;
            writeln!(&mut res, "{json_text}")?;
            Ok(())
        })?;
    }

    // Keep server running beyond when main() returns (forever)
    // Do not call this if you ever want to stop or access it later.
    // Otherwise you can either add an infinite loop so the main task
    // never returns, or you can move it to another thread.
    // https://doc.rust-lang.org/stable/core/mem/fn.forget.html
    std::mem::forget(wifi);
    std::mem::forget(server);

    Ok(())
}

fn create_server<'a>() -> Result<EspHttpServer<'a>> {
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };

    // Keep wifi running beyond when this function returns (forever)
    // Do not call this if you ever want to stop or access it later.
    // Otherwise it should be returned from this function and kept somewhere
    // so it does not go out of scope.
    // https://doc.rust-lang.org/stable/core/mem/fn.forget.html

    Ok(EspHttpServer::new(&server_configuration)?)
}

fn connect_wifi<M>(
    ssid: &str,
    pass: &str,
    modem: impl Peripheral<P = M> + 'static,
    sys_loop: EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
) -> Result<BlockingWifi<EspWifi<'static>>>
where
    M: WifiModemPeripheral,
{
    use esp_idf_svc::wifi::{ClientConfiguration, Configuration};

    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), nvs)?, sys_loop)?;

    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: pass.try_into().unwrap(),
        channel: None,
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    log::info!("Wifi started");

    wifi.connect()?;
    log::info!("Wifi connected");

    wifi.wait_netif_up()?;
    log::info!("Wifi netif up");

    Ok(wifi)
}
