mod influxdb;
mod portfolio;
mod quote_producer;
mod quote_receiver;
mod yahoo_api;
mod yahoo;

use anyhow::anyhow;

use crate::yahoo::YahooFinanceQuote;

use futures_util::{try_join, TryFutureExt};
use pin_utils::pin_mut;
use tokio::sync::mpsc;

use clap::Parser;
use serde::Deserialize;

use crate::influxdb::Config as InfluxDBConfig;
use crate::influxdb::{InfluxDB, Measurement};
use crate::portfolio::Currency::EUR;
use crate::portfolio::{Currency, Portfolio, Quote, QuoteMeta};
use crate::yahoo_api::Yahoo;
use quote_producer::QuoteProducer;
use quote_receiver::QuoteReceiver;
use tokio_tungstenite::tungstenite::Message;

#[derive(Parser)]
struct Cli {
    #[clap(long, short)]
    file: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    home_currency: Currency,
    portfolio: Vec<(String, f64)>,
    db: Option<InfluxDBConfig>,
    print_portfolio: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            print_portfolio: true,
            home_currency: EUR,
            db: None,
            portfolio: vec![
                ("BTC-USD".into(), 10.),
                ("AETH-USD.SW".into(), 450.),
                ("AMZN".into(), 1.),
                ("DE000A27Z304.SG".into(), 500.),
                ("CSNDX.SW".into(), 10.),
                ("EXS2.DE".into(), 10.),
                ("IBCL.DE".into(), 10.),
                ("ITEK.MI".into(), 2000.),
                ("IUIT.SW".into(), 2000.),
                ("IUSE.SW".into(), 100.),
                ("MSFT".into(), 25.),
                ("TSM".into(), 100.),
                ("XDWT.DE".into(), 1000.),
            ],
        }
    }
}

fn cls() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

pub type QuoteMessage = Vec<Quote>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let config = match cli.file {
        Some(path) => {
            let json_data = std::fs::read_to_string(path).expect("Unable to open the file");
            serde_json::from_str(&json_data).expect("Unable to deserialize json")
        },
        None => Config::default()
    };

    if !matches!(config.home_currency, EUR) {
        panic!("Only EUR supported as home currency for now");
    }

    let (tx, rx) = mpsc::channel::<QuoteMessage>(32);

    let c_config = config.clone();

    let producer = tokio::spawn(async move {
        let quote_producer = QuoteProducer::new(c_config, tx);

        quote_producer.start().await
    }).map_err(|e| anyhow!(e)).and_then(|x| async { x });

    let c_config = config.clone();

    let receiver = tokio::spawn(async move {
        let mut quote_receiver = QuoteReceiver::new(c_config, rx);
        
        quote_receiver.start().await
    }).map_err(|e| anyhow!(e)).and_then(|x| async { x });

    pin_mut!(producer);
    pin_mut!(receiver);

    try_join!(receiver, producer)?;

    Ok(())
}
