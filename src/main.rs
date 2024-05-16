use std::fs::File;

use futures_util::{pin_mut, StreamExt};
use log::{debug, error, info, LevelFilter};

mod aevo;
mod arbitrage;
mod dydx;
mod socket_stream;

use aevo::AevoBuilder;
use arbitrage::Arbitrage;
use dydx::DyDxBuilder;

#[tokio::main]
async fn main() {
    // set the logger environment
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    // initialize aevo and dydx connections/streams
    let mut aevo_builder = AevoBuilder::new();
    let mut dydx_builder = DyDxBuilder::new();

    let Ok(mut aevo) = aevo_builder.connect().await else {
        error!("Could not connect to Aevo, aborting...");
        return;
    };

    let Ok(mut dydx) = dydx_builder.connect().await else {
        error!("Could not connect to DyDx, aborting...");
        return;
    };

    let aevo_stream = aevo.listen();
    let dydx_stream = dydx.listen();

    // create a file to keep track of transactions and profits
    let mut file = File::create("trades.txt")
        .expect("maybe the user does not have the permission to create files");

    // thank you rust!
    pin_mut!(dydx_stream);
    pin_mut!(aevo_stream);

    // initialize arbitrage simulation settings
    let dydx_balance = 1000.0;
    let aevo_balance = 1000.0;
    let fees = 0.0;
    let max_trade_amount = 200.0;

    let mut arbitrage_bot = Arbitrage::new(dydx_balance, aevo_balance, fees, max_trade_amount);

    // listen to both streams concurrently and simulate the arbitrage scenario
    while arbitrage_bot.dydx_balance > 200.0 && arbitrage_bot.aevo_balance > 200.0 {
        tokio::select! {
            Some(dydx_price) = dydx_stream.next() => {
                info!("DyDx Price: {}", dydx_price);
                arbitrage_bot.dydx_btc_price = Some(dydx_price);
                if let Some(aevo_price) = arbitrage_bot.aevo_btc_price {
                    debug!("also found aevo price: {}", aevo_price);
                    arbitrage_bot.simulate_arbitrage( aevo_price, dydx_price, &mut file).await;
                }
            },
            Some(aevo_price) = aevo_stream.next() => {
                info!("Aevo Price: {}", aevo_price);
                arbitrage_bot.aevo_btc_price = Some(aevo_price);
                if let Some(dydx_price) = arbitrage_bot.dydx_btc_price {
                    debug!("also found dydx price: {}", dydx_price);
                    arbitrage_bot.simulate_arbitrage(aevo_price, dydx_price, &mut file).await;
                }
            }
        }

        // aevo stream sends the price very frequently compared to DyDx, due to DyDx sends them in batch
        // so, it's better to wait for a few seconds to give both channels enough time to compete
        // this way, it will be more realistic
        // we can just remove this, and it will still work though :)
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
