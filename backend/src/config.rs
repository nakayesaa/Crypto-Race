use std::env;

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
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let trading_pairs = env::var("TRADING_PAIRS")
            .expect("TRADING_PAIRS missing from .env")
            .split(',')
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

        Ok(Self {
            time_window,
            port,
            trading_pairs,
            broadcast_interval_ms,
        })
    }
}
