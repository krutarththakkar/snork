mod eatall;
pub use eatall::EatAllAgent;
mod random;
pub use random::Random;
mod util;

use super::env::{GameRequest, MoveResponse};

pub trait Agent: std::fmt::Debug + 'static {
    fn start(&mut self, request: &GameRequest);
    fn step(&mut self, request: &GameRequest) -> MoveResponse;
    fn end(&mut self, request: &GameRequest);
}
