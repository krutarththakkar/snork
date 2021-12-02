use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};

mod mobility;
pub use mobility::*;
mod flood;
pub use flood::*;
mod random;
pub use random::RandomAgent;
mod tree;
pub use tree::*;
mod mcts;
pub use mcts::*;

use super::env::{GameRequest, MoveResponse};

pub trait Agent: std::fmt::Debug + 'static {
    fn step(&mut self, request: &GameRequest, ms: u64) -> MoveResponse;
    fn end(&mut self, request: &GameRequest);
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Config {
    Mobility(MobilityConfig),
    Tree(TreeConfig),
    Random,
    Flood(FloodConfig),
}

impl Default for Config {
    fn default() -> Self {
        Config::Tree(TreeConfig::default())
    }
}

impl Config {
    pub fn create_agent(&self, request: &GameRequest) -> Arc<Mutex<dyn Agent + Send>> {
        if request.board.width > 19 || request.board.height > 19 {
            return Arc::new(Mutex::new(RandomAgent::default()));
        }

        match self {
            Config::Mobility(config) => Arc::new(Mutex::new(MobilityAgent::new(request, &config))),
            Config::Tree(config) => Arc::new(Mutex::new(TreeAgent::new(request, &config))),
            Config::Flood(config) => Arc::new(Mutex::new(FloodAgent::new(request, &config))),
            _ => Arc::new(Mutex::new(RandomAgent::default())),
        }
    }
}

impl FromStr for Config {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ToString for Config {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
