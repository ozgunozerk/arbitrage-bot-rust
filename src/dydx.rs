use anyhow::{anyhow, Result};
use futures_core::Stream;
use serde_json::json;

use super::socket_stream::*;

pub struct DyDxBuilder {
    pub url: String,
    pub subscribe_message: String,
}

pub struct DyDx {
    socket: SocketStream,
}

impl DyDxBuilder {
    /// creates a new builder object with the url and websocket subscription message
    pub fn new() -> Self {
        let subscribe_message = json!({
            "type": "subscribe",
            "channel": "v4_trades",
            "id": "ETH-USD"
        })
        .to_string();

        DyDxBuilder {
            url: "wss://indexer.dydx.trade/v4/ws".to_string(),
            subscribe_message,
        }
    }

    /// connects to DyDx WebSocket server and returns the DyDx object with socket
    pub async fn connect(&mut self) -> Result<DyDx> {
        if let Ok(socket) = <DyDxBuilder as WebSocketBuilder>::get_websocket_stream(
            self.url.to_owned(),
            self.subscribe_message.to_owned(),
        )
        .await
        {
            Ok(DyDx { socket })
        } else {
            Err(anyhow!("Could not connect to DyDx"))
        }
    }
}

impl DyDx {
    /// listens to DyDx WebSocket server and parses the received messages
    pub fn listen(&mut self) -> impl Stream<Item = <DyDx as WebSocketConnection>::Price> + '_ {
        <DyDx as WebSocketConnection>::listen_to_stream(&mut self.socket)
    }
}

impl WebSocketBuilder for DyDxBuilder {}

impl WebSocketConnection for DyDx {
    type Price = f64;

    fn handle_message(text: &str) -> Result<Self::Price> {
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(json) => {
                let Some(trades) = json["contents"]["trades"].as_array() else {
                    return Err(anyhow!("Trades field is missing or is not an array"));
                };
                let Some(last_trade) = trades.last() else {
                    return Err(anyhow!("Trades array is empty"));
                };
                let Some(price) = last_trade["price"].as_str() else {
                    return Err(anyhow!("Price field is missing in the last trade"));
                };
                Ok(price.parse::<f64>()?)
            }
            Err(_) => Err(anyhow!("Failed to parse the message: {}", text)),
        }
    }
}
