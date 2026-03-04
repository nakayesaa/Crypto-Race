use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod config;
mod models;
mod coingecko;
mod price_engine;
mod ws_server;

use config::Config;
use price_engine::Pricestore;
use ws_server::ClientRegistry;

#[tokio::main] // tokio runtime for async main function
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let config = Config::from_env().expect("Load Config Failed");
    tracing::info!("Load Config: {:?}", config);

    let config = Arc::new(config);
    let price_store = Arc::new(Mutex::new(Pricestore::new()));
    let registry: ClientRegistry = Arc::new(Mutex::new(Vec::new()));

    // Spawn the CoinGecko poller task
    let coingecko_config = Arc::clone(&config);
    let coingecko_store = Arc::clone(&price_store);
    tokio::spawn(async move {
        coingecko::run(coingecko_config, coingecko_store).await;
    });
    
    // Spawn the broadcast loop
    let broadcast_config = Arc::clone(&config);
    let broadcast_store = Arc::clone(&price_store);
    let broadcast_registry = Arc::clone(&registry);
    tokio::spawn(async move {
        let mut positions: HashMap<String, f64> = HashMap::new();
        let dt = broadcast_config.broadcast_interval_ms as f64 / 1000.0;

        loop {
            tokio::time::sleep(Duration::from_millis(broadcast_config.broadcast_interval_ms)).await;

            // Compute race state from the price store
            let mut race_state = {
                let store = broadcast_store.lock().unwrap();
                price_engine::compute_race_state(
                    &store,
                    &broadcast_config.time_window,
                    &broadcast_config,
                )
            };

            // Update positions: position += speed * dt, wrap at 1.0
            for car in &mut race_state.cars {
                if car.speed.is_nan() || car.speed.is_infinite() {
                    car.position = *positions.entry(car.symbol.clone()).or_insert(0.0);
                    continue;
                }
                let pos = positions.entry(car.symbol.clone()).or_insert(0.0);
                *pos += car.speed * dt;
                if *pos > 1.0 {
                    *pos -= 1.0;
                }
                car.position = *pos;
            }

            // Serialize and broadcast to all connected clients
            let json = serde_json::to_string(&race_state).unwrap();
            let mut clients = broadcast_registry.lock().unwrap();
            // .retain keeps only clients where send succeeds (removes dead ones)
            clients.retain(|tx| tx.send(json.clone()).is_ok());
        }
    });

    // Start the Axum HTTP/WS server
    let app = ws_server::create_router(Arc::clone(&registry));
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
