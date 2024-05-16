use anyhow::{anyhow, Result};
use async_stream::stream;
use futures_core::stream::Stream;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub type SocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub trait WebSocketBuilder {
    /// connects to the websocket, and returns a WebSocketStream
    async fn get_websocket_stream(url: String, subscribe_message: String) -> Result<SocketStream> {
        info!("Connecting to {}", url);

        // Connect to the WebSocket server
        match connect_async(url).await {
            Ok((mut ws_stream, _)) => {
                info!("Connected to the WebSocket server");

                // Send the subscription message
                if let Err(e) = ws_stream.send(Message::Text(subscribe_message)).await {
                    return Err(anyhow!("Failed to send subscription message: {}", e));
                }
                info!("Subscription message sent");

                Ok(ws_stream)
            }
            Err(e) => Err(anyhow!("Failed to connect: {}", e)),
        }
    }
}

pub trait WebSocketConnection {
    type Price;

    /// method to listen to the websocket and receive messages
    /// calls `handle_message` internally to parse the received messages
    fn listen_to_stream(ws_stream: &mut SocketStream) -> impl Stream<Item = Self::Price> {
        stream! {
            // listen to the incoming messages and parse them
            while let Some(message) = ws_stream.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Ok(price) = Self::handle_message(&text) {
                            yield price;
                        } else {
                            continue;
                        }
                    },
                    Ok(non_text) => {
                        debug!("received non-text message: {}", non_text);
                        continue;
                    },
                    Err(e) => {
                        error!("Error during the WebSocket communication: {}", e);
                        continue;
                    },
                }

            }
        }
    }

    // method to handle incoming WebSocket messages
    fn handle_message(text: &str) -> Result<Self::Price>;
}
