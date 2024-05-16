use anyhow::{anyhow, Result};
use futures_core::Stream;
use serde_json::json;

use super::socket_stream::*;

pub struct AevoBuilder {
    pub url: String,
    pub subscribe_message: String,
}

impl AevoBuilder {
    /// creates a new builder object with the url and websocket subscription message
    pub fn new() -> Self {
        let subscribe_message = json!({
            "op": "subscribe",
            "data": ["ticker:ETH:PERPETUAL"]
        })
        .to_string();

        AevoBuilder {
            url: "wss://ws.aevo.xyz".to_string(),
            subscribe_message,
        }
    }

    /// connects to Aevo WebSocket server and returns the Aevo object with socket
    pub async fn connect(&mut self) -> Result<Aevo> {
        if let Ok(socket) = <AevoBuilder as WebSocketBuilder>::get_websocket_stream(
            self.url.to_owned(),
            self.subscribe_message.to_owned(),
        )
        .await
        {
            Ok(Aevo { socket })
        } else {
            Err(anyhow!("Could not connect to DyDx"))
        }
    }
}

impl WebSocketBuilder for AevoBuilder {}

pub struct Aevo {
    pub socket: SocketStream,
}

impl Aevo {
    /// listens to Aevo WebSocket server and parses the received messages
    pub fn listen(&mut self) -> impl Stream<Item = <Aevo as WebSocketConnection>::Price> + '_ {
        <Aevo as WebSocketConnection>::listen_to_stream(&mut self.socket)
    }
}

impl WebSocketConnection for Aevo {
    type Price = f64;

    fn handle_message(text: &str) -> Result<Self::Price> {
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(json) => {
                let Some(tickers) = json["data"]["tickers"].as_array() else {
                    return Err(anyhow!("Trades field is missing or is not an array"));
                };
                let Some(ticker) = tickers.first() else {
                    return Err(anyhow!("Trades array is empty"));
                };
                let Some(price) = ticker["index_price"].as_str() else {
                    return Err(anyhow!("Price field is missing in the last trade"));
                };
                Ok(price.parse::<f64>()?)
            }
            Err(_) => Err(anyhow!("Failed to parse the message: {}", text)),
        }
    }
}
