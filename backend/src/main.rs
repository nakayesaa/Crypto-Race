mod config;
use config::Config;

fn main() {
    let config = Config::from_env().expect("config load failed");
    println!("{:?}", config);
}
