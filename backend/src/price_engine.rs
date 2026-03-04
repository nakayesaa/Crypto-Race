use std::collections::{HashMap, VecDeque};
use chrono::{Utc, Duration};
use crate::models::{CarState, PriceTick, RaceState};
use crate::config::{Config, TimeWindow};

pub struct Pricestore{
    price_store : HashMap<String, VecDeque<PriceTick>>
}

impl Pricestore{
    pub fn new() -> Self{
        Self{
            price_store : HashMap::new()
        }
    }

    pub fn push_tick(&mut self, tick:PriceTick){
        let deque = self
            .price_store
            .entry(tick.symbol.clone())
            .or_insert_with(VecDeque::new);
        
        deque.push_back(tick);
        let cut = Utc::now() - Duration::hours(24);

        while let Some(front) = deque.front() {
            if front.timestamp < cut{
                deque.pop_front();
            }else{
                break;
            } 
        }
    }
}

fn compute_percent_change(store: &Pricestore, symbol: &str, window: &TimeWindow) -> Option<f64> {
    let deque = store.price_store.get(symbol);
    if let Some(deque) = deque {
        if deque.is_empty() {
            return None;
        }
        if deque.len() == 1 {
            return Some(0.0);
        }
        let current = deque.back().unwrap().price;
        let duration = match window {
            TimeWindow::M1  => Duration::minutes(1),
            TimeWindow::M5  => Duration::minutes(5),
            TimeWindow::M15 => Duration::minutes(15),
            TimeWindow::H1  => Duration::hours(1),
            TimeWindow::H24 => Duration::hours(24),
        };
        let cut = Utc::now() - duration;
        // Find the oldest tick within (or before) the window.
        // If no tick is older than the cutoff, fall back to the oldest available tick
        // so we still get a meaningful % change at startup.
        let open_tick = deque.iter().rev()
            .find(|tick| tick.timestamp <= cut)
            .or_else(|| deque.front());
        if let Some(open_tick) = open_tick{
            let open = open_tick.price;
            if open == 0.0{
                return None;
            }
            Some((current - open) / open * 100.0)
        }else{
            None
        }
    }else {
        return None;
    }
}

const CAR_COLORS: &[&str] = &[
    "#e63946", "#f4a261", "#31625c", "#457b9d",
    "#8338ec", "#fb5607", "#06d6a0", "#ffd166",
];

pub fn compute_race_state(store: &Pricestore, window: &TimeWindow, config: &Config) -> RaceState {
    let mut percentage_map: HashMap<String, f64> = HashMap::new();
    for symbol in &config.coin_ids{
        if let Some(percentage) = compute_percent_change(store, symbol, window){
            percentage_map.insert(symbol.clone(), percentage);
        }
    }

    let minimum_percentage = percentage_map.values().cloned().fold(f64::INFINITY, f64::min);
    let maximum_percentage = percentage_map.values().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if percentage_map.is_empty() { 0.0 } else { maximum_percentage - minimum_percentage };

    let time_window_str = match window {
        TimeWindow::M1  => "1m",
        TimeWindow::M5  => "5m",
        TimeWindow::M15 => "15m",
        TimeWindow::H1  => "1h",
        TimeWindow::H24 => "24h",
    }.to_string();

    let cars = config.coin_ids.iter().enumerate().map(|(i, symbol)|{
        let percentage = percentage_map.get(symbol).copied().unwrap_or(0.0);

        let speed = if range == 0.0{
            0.9
        }else{
            0.3 + (percentage - minimum_percentage) / range * 1.2
        };  

        let price = store.price_store
            .get(symbol)
            .and_then(|d| d.back())
            .map(|t| t.price)
            .unwrap_or(0.0);

        CarState {
            symbol: symbol.clone(),
            display_name: symbol.clone(),
            price,
            percent_change: percentage,
            speed,
            position: 0.0,
            color_hex: CAR_COLORS[i % CAR_COLORS.len()].to_string(),
        }
    }).collect();

    RaceState {
        timestamp: Utc::now(),
        time_window: time_window_str,
        cars,
    }
}
