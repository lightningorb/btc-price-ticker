use std::fs;
use anyhow::Context;
use lazy_static::lazy_static;
use log::error;
use serde_json::Value;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tungstenite::{connect, Message};
use url::Url;
use serde::Deserialize;
use std::path::PathBuf;
use dirs::home_dir;


#[derive(Deserialize)]
struct Config {
    average_price_in_last_seconds: u64
}

lazy_static! {
    static ref CONFIG_FILE: PathBuf = {
        let mut path = home_dir().expect("Could not find home directory");
        path.push(".config/orb_price_ticker/config.toml");
        path
    };
}


lazy_static! {
    static ref PRICES: Arc<Mutex<Vec<(u64, f64)>>> = Arc::new(Mutex::new(Vec::new()));
}

fn handle_stream(mut unix_stream: UnixStream, average_price_in_last_seconds: u64) -> anyhow::Result<()> {
    let mut message = String::new();
    unix_stream
        .read_to_string(&mut message)
        .context("Failed at reading the unix stream")?;

    println!("We received this message: {}\nReplying...", message);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let avg_price = {
        let prices_lock = PRICES.lock().unwrap();
        if !prices_lock.is_empty() {
            calculate_rolling_average(prices_lock.as_slice(), now, average_price_in_last_seconds)
        } else {
            0.0
        }
    };

    let response_message = format!("{:.2}", avg_price);

    unix_stream
        .write(response_message.as_bytes())
        .context("Failed at writing onto the unix stream")?;

    Ok(())
}

fn calculate_rolling_average(prices: &[(u64, f64)], current_time: u64, average_price_in_last_seconds: u64) -> f64 {
    let window_duration = average_price_in_last_seconds;
    let filtered_prices: Vec<f64> = prices.iter()
        .filter(|&&(timestamp, _)| current_time - timestamp < window_duration)
        .map(|&(_, price)| price)
        .collect();

    let sum: f64 = filtered_prices.iter().sum();
    let count = filtered_prices.len() as f64;
    
    if count == 0.0 {
        0.0
    } else {
        sum / count
    }
}


#[tokio::main]
async fn main() {
    let config_contents = fs::read_to_string(CONFIG_FILE.as_path())
        .expect("Failed to read config.toml");
    let config: Config = toml::from_str(&config_contents)
        .expect("Invalid TOML format in config.toml");
    let socket_path = "/tmp/mysocket";

    if std::fs::metadata(socket_path).is_ok() {
        println!("A socket is already present. Deleting...");
        let _ = std::fs::remove_file(socket_path).with_context(|| {
            format!("could not delete previous socket at {:?}", socket_path)
        });
    }

    let unix_listener =
        UnixListener::bind(socket_path).context("Could not create the unix socket");

    thread::spawn(move || {
        for stream in unix_listener.expect("REASON").incoming() {
            match stream {
                Ok(unix_stream) => {
                    if let Err(e) = handle_stream(unix_stream, config.average_price_in_last_seconds) {
                        eprintln!("Error handling stream: {}", e);
                    }
                }
                Err(err) => {
                    eprintln!("Server loop error: {}", err);
                    break;
                }
            }
        }
    });

    let binance_ws_api: &str = "wss://stream.binance.com:9443/ws/btcusdt@trade";

    loop {
        match connect(Url::parse(binance_ws_api).unwrap()) {
            Ok((mut socket, _response)) => {
                loop {
                    let result = timeout(Duration::from_secs(10), async {
                        socket.read()
                    })
                    .await;

                    match result {
                        Ok(Ok(Message::Text(text_msg))) => {
                            let mut prices_guard = PRICES.lock().unwrap();
                            on_message(&text_msg, &mut *prices_guard, config.average_price_in_last_seconds);
                            drop(prices_guard);
                        }
                        
                        Ok(Err(e)) => {
                            error!("Error reading message: {:?}", e);
                            break;
                        }
                        Err(_) => {
                            error!("WebSocket timed out waiting for message");
                            break;
                        }
                        _ => {
                            error!("Unexpected message type or error.");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Cannot connect to WebSocket: {:?}", e);
            }
        }

        error!("Reconnecting to WebSocket...");
        sleep(Duration::from_secs(5)).await;
    }
}

pub fn on_message(message: &str, prices: &mut Vec<(u64, f64)>, average_price_in_last_seconds: u64) {
    if let Ok(parsed) = serde_json::from_str::<Value>(message) {
        if let Some(price_str) = parsed["p"].as_str() {
            if let Ok(price) = price_str.parse::<f64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                prices.push((now, price));
                let sixty_seconds_ago = now - average_price_in_last_seconds;
                prices.retain(|&(t, _)| t > sixty_seconds_ago);
            }
        }
    } else {
        error!("Failed to parse message as JSON");
    }
}
