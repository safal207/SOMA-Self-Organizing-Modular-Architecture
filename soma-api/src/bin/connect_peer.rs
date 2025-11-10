use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: connect_peer <source_port> <target_port>");
        println!("Example: connect_peer 8081 8080");
        std::process::exit(1);
    }

    let source_port = &args[1];
    let target_port = &args[2];
    let url = format!("ws://127.0.0.1:{}/mesh", target_port);

    println!("📡 Connecting node on port {} to {}", source_port, url);

    match connect_async(&url).await {
        Ok((ws_stream, _)) => {
            println!("✅ Connected! Mesh link established.");
            let (mut write, mut read) = ws_stream.split();

            // Слушаем сообщения
            tokio::spawn(async move {
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(txt)) => {
                            println!("📥 Received: {}", txt);
                        }
                        Ok(Message::Close(_)) => {
                            println!("🔌 Connection closed");
                            break;
                        }
                        Err(e) => {
                            println!("❌ Error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            });

            // Держим соединение открытым
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            std::process::exit(1);
        }
    }
}
