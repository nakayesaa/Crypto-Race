use std::env;
//this file is responsible for loading configuration from environtment variables   
// and providing a Config struct that the rest of the backend can use

#[derive(Debug)]
pub enum TimeWindow {
    M1,
    M5,
    M15,
    H1,
    H24,
}

#[derive(Debug)]
pub struct Config {
    pub time_window: TimeWindow,
    pub port: u16,
    pub trading_pairs: Vec<String>,
    pub broadcast_interval_ms: u64,
    pub coin_ids : Vec<String>,
    pub coingecko_api_key : Option<String>,
    pub poll_interval_ms: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();
        let trading_pairs = env::var("TRADING_PAIRS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|value| value.trim().to_string())
            .collect();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "9001".into())
            .parse::<u16>()
            .map_err(|_| "PORT must be a number".to_string())?;

        let broadcast_interval_ms = env::var("BROADCAST_INTERVAL_MS")
            .unwrap_or_else(|_| "500".into())
            .parse::<u64>()
            .map_err(|_| "BROADCAST_INTERVAL_MS must be a number".to_string())?;

        let time_window = match env::var("DEFAULT_TIME_WINDOW")
            .unwrap_or_else(|_| "1h".into())
            .as_str()
        {
            "1m"  => TimeWindow::M1,
            "5m"  => TimeWindow::M5,
            "15m" => TimeWindow::M15,
            "1h"  => TimeWindow::H1,
            "24h" => TimeWindow::H24,
            other => return Err(format!("Unknown DEFAULT_TIME_WINDOW: {}", other)),
        };

        let coin_ids = env::var("COIN_IDS")
            .expect("COIN_IDS missing from .env")
            .split(',')
            .map(|value| value.trim().to_string())
            .collect();

        let coingecko_api_key = env::var("COINGECKO_API_KEY").ok();

        let poll_interval_ms = env::var("POLL_INTERVAL_MS")
            .unwrap_or_else(|_| "60000".into())
            .parse::<u64>()
            .map_err(|_| "POLL_INTERVAL_MS must be a number".to_string())?;

        Ok(Self {
            time_window,
            port,
            trading_pairs,
            coin_ids,
            broadcast_interval_ms,
            coingecko_api_key,
            poll_interval_ms
        })
    }
}
