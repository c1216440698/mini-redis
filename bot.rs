mod errors;
use dashmap::DashMap;
use polyfill_rs::errors::PolyfillError;
use polyfill_rs::{ClobClient, OrderType, Side};
use polyfill_rs::{OrderBookImpl, WebSocketStream};
use serde_json::Value;
use std::collections::HashMap;
use std::{sync::Arc, task::Context};

pub const EVENT_URL: &'static str = "https://gamma-api.polymarket.com/events";
pub const SLUG_URL: &'static str = "https://gamma-api.polymarket.com/events/slug";
pub const MARKET_URL: &'static str = "https://gamma-api.polymarket.com/markets";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MarketType {
    POLYMARKET,
    KALSHI,
}

#[derive(Debug, Clone, Hash)]
enum Outcome {
    YES,
    NO,
}

#[derive(Debug, Clone, Hash)]
struct Token {
    market_name: String,
    token_id: u64,
    outcome: Outcome,
}

impl Token {
    fn new(name: String, id: u64, o: Outcome) -> Self {
        Self {
            market_name: name,
            token_id: id,
            outcome: o,
        }
    }
}

struct TokenFetch {
    context: Arc<BotConext>,
    tokens: Vec<Token>,
}

pub type Result<T> = std::result::Result<T, PolyfillError>;

pub trait TokenTrait<'a, 'b> {
    async fn get_events_by_params(&mut self, params: HashMap<&'a str, &'a str>) -> Result<Value>;
}

impl<'a, 'b> TokenTrait<'a, 'b> for ClobClient {
    async fn get_events_by_params(&mut self, params: HashMap<&'a str, &'a str>) -> Result<Value> {
        let response = self
            .http_client
            .get(EVENT_URL)
            .json(&params)
            .send()
            .await
            .map_err(|e| PolyfillError::network(format!("Request failed: {}", e), e))?;

        response
            .json::<Value>()
            .await
            .map_err(|e| PolyfillError::parse(format!("Parse response failed: {}", e), None))
    }
}

impl TokenFetch {
    fn new() -> Self {
        Self {
            context: Arc::new(BotConext::new()),
            tokens: Vec::with_capacity(200),
        }
    }

    async fn get_live_sports_tokens(&mut self) {}

    async fn get_crypto_tokens(&mut self) {}
}

struct BotConext {
    client: Arc<ClobClient>,
    tokens: DashMap<MarketType, Vec<Token>>,
}

impl BotConext {
    fn new() -> Self {
        Self {
            client: Arc::new(ClobClient::new("127.0.0.1")),
            tokens: DashMap::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClobClient::new("https://clob.polymarket.com");
    Ok(())
}
