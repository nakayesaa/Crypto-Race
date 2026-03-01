use std::collections::{HashMap, VecDeque};
use chrono::{Utc, Duration};
use crate::models::PriceTick;
use crate::config::TimeWindow;

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
        let open_tick = deque.iter().rev().find(|tick| tick.timestamp <= cut);
        if let Some(open_tick) = open_tick{
            let open = open_tick.price;
            if open == 0.0{
                return None;
            }
            return Some((current - open) / open * 100.0);
        }else{
            return None;
        }
    }else {
        return None;
    }
}