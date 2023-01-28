use crate::game::movement::OrderMove;
use crate::game::Board;
use std::sync::{Arc, RwLock};

// #[derive(Clone)]
pub struct AppState {
    board: Arc<RwLock<Board>>,
    order_move: Box<dyn OrderMove>,
}
