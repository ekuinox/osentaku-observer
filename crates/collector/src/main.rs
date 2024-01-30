use std::{
    fs::File,
    path::PathBuf,
    time::{Duration, UNIX_EPOCH},
};

use anyhow::{Context as _, Result};
use clap::Parser;
use schema::Acceleration;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

#[derive(Parser, Debug)]
pub struct Cli {
    addr: String,

    #[clap(short, long)]
    out: Option<PathBuf>,

    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let fetcher = AccelFetcher::new(&cli.addr);

    let path = match cli.out {
        Some(out) if out.is_dir() => out.join(format!("{}.csv", ts())),
        Some(out) => out,
        None => PathBuf::new().join(format!("{}.csv", ts())),
    };

    let file = File::create(&path).expect("Failed to create output file.");
    let mut writer = csv::WriterBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_writer(file);

    println!("out = {path:?}");

    loop {
        let now = Instant::now();

        if let Ok(data) = fetcher.fetch().await {
            let row = CsvRow::now(&data);
            if cli.verbose {
                println!("{row:?}");
            }
            writer.serialize(&row).expect("Failed to write csv row.");
            writer.flush().expect("Failed to flush csv row.");
        }
        tokio::time::sleep_until(now + Duration::from_millis(1000)).await;
    }
}

struct AccelFetcher {
    url: String,
}

impl AccelFetcher {
    pub fn new(addr: &str) -> AccelFetcher {
        AccelFetcher {
            url: format!("http://{addr}/accel"),
        }
    }

    pub async fn fetch(&self) -> Result<Acceleration> {
        let res = reqwest::get(&self.url)
            .await
            .context("Failed to fetch data.")?
            .json::<Acceleration>()
            .await
            .context("Failed to parse data.")?;
        Ok(res)
    }
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
struct CsvRow {
    pub ts: u128,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl CsvRow {
    pub fn now(accel: &Acceleration) -> Self {
        CsvRow {
            ts: ts(),
            x: accel.x,
            y: accel.y,
            z: accel.z,
        }
    }
}

fn ts() -> u128 {
    UNIX_EPOCH.elapsed().unwrap().as_millis()
}
