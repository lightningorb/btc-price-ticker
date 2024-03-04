use std::time::SystemTime;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use num_format::{Locale, ToFormattedString};
use serde::Deserialize;
use std::fs;
use anyhow::Context;
use dirs::home_dir;
use std::path::PathBuf;
use lazy_static::lazy_static;
use std::time::Duration;
use tokio::time::Instant;

#[derive(Deserialize)]
struct Config {
    enable_portfolio_mode: bool,
    portfolio_value_in_btc: f32,
    round: bool
}

lazy_static! {
    static ref CONFIG_FILE: PathBuf = {
        let mut path = home_dir().expect("Could not find home directory");
        path.push(".config/orb_price_ticker/config.toml");
        path
    };
}

fn main() -> anyhow::Result<()> {
    let config_contents = fs::read_to_string(CONFIG_FILE.as_path())
        .expect("Failed to read config.toml");
    let config: Config = toml::from_str(&config_contents)
        .expect("Invalid TOML format in config.toml");
    let socket_path = "/tmp/mysocket";
    let mut unix_stream = connect_with_timeout(socket_path, Duration::from_millis(10))?;

    write_request_and_shutdown(&mut unix_stream)?;
    read_from_stream(&mut unix_stream, config)?;
    Ok(())
}

fn connect_with_timeout(path: &str, timeout: Duration) -> io::Result<UnixStream> {
    let start = Instant::now();
    loop {
        match UnixStream::connect(path) {
            Ok(stream) => return Ok(stream),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if start.elapsed() >= timeout {
                    return Err(io::Error::new(io::ErrorKind::TimedOut, "Connection timed out"));
                }
            }
            Err(e) => return Err(e),
        }
    }
}

fn write_request_and_shutdown(unix_stream: &mut UnixStream) -> anyhow::Result<()> {
    unix_stream
        .write(b"Hello?")
        .context("Failed at writing onto the unix stream")?;

    unix_stream
        .shutdown(std::net::Shutdown::Write)
        .context("Could not shutdown writing on the stream")?;

    Ok(())
}


fn read_from_stream(unix_stream: &mut UnixStream, config: Config) -> anyhow::Result<()> {
    let mut response = String::new();
    unix_stream
        .read_to_string(&mut response)
        .context("Failed at reading the unix stream")?;

    let price: f32 = response.parse::<f32>().unwrap();

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!");
    let current_minute = (current_time.as_secs() / 60) % 60;

    if config.enable_portfolio_mode {
        let portfolio_value_in_btc = config.portfolio_value_in_btc;
        let portfolio_value = price * portfolio_value_in_btc;

        if current_minute % 2 == 0 {
            println!("${}", format_price(price, config.round));
        } else {
            println!("${}", format_price(portfolio_value, config.round));
        }
    } else {
        println!("Price: ${}", format_price(price, config.round));
    }

    Ok(())
}

// Helper function to format the price or portfolio value
fn format_price(value: f32, round: bool) -> String {
    if round {
        (value as i32).to_formatted_string(&Locale::en)
    } else {
        let integral_part = value.trunc() as i64;
        let fractional_part = (value.fract() * 100.0).round() as i64;
        let formatted_integral = integral_part.to_formatted_string(&Locale::en);
        format!("{}.{}", formatted_integral, format!("{:02}", fractional_part))
    }
}