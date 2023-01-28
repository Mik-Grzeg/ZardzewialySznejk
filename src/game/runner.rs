use self::movement::OrderError;

use super::board::{Board, CellSymbol};
use super::consts::*;
use super::point::Direction;
use super::snake::{Snake, SnakeError};
use std::collections::HashMap;
use std::sync::RwLock;

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{interval_at, Duration, Instant, Interval, MissedTickBehavior};
use tracing::{debug, error, info, trace, warn};
use movement::OrderMove;

const MOVE_COMMAND_CHANNEL_SIZE: usize = 1000;

pub mod movement {
    use std::fmt::Debug;
    use async_trait::async_trait;
    use tokio::sync::mpsc::error::SendError;
    use super::Direction;
    use thiserror::Error;

    #[async_trait]
    pub trait OrderMove: Send + Sync + Debug {
        async fn issue_move(&self, direction: Direction) -> Result<(), OrderError>;
    }

    #[derive(Error, Debug)]
    pub enum OrderError {
        #[error("Unable to issue new movement command `{0}`")]
        IssueMovement(String)
    }

    impl<T: Debug> From<SendError<T>> for OrderError {
        fn from(send_err: SendError<T>) -> Self {
            Self::IssueMovement(send_err.to_string())
        }
    }
}

enum MoveCommandManager {
    Issuer(MoveCommandIssuer),
    Receiver(MoveCommandReceiver),
}

#[derive(Debug)]
struct MoveCommandReceiver {
    command_rx: Receiver<Direction>,
}

impl From<Receiver<Direction>> for MoveCommandReceiver {
    fn from(command_rx: Receiver<Direction>) -> Self {
        Self { command_rx }
    }
}

impl MoveCommandReceiver {
    async fn wait_for_command_and_act(&mut self, direction_command_counters: &mut HashMap<Direction, u32>, current_direction: &Direction) {
        match self.command_rx.recv().await {
            Some(c) => {
                if c == current_direction.opposite() {
                    return
                }
                direction_command_counters.entry(c).and_modify(|counter| *counter +=  1).or_insert(1);
                debug!("Received a move command from user {:?}", c);
            }
            None => {
                warn!("Received a move command although it was empty");
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveCommandIssuer {
    command_sender: Sender<Direction>,
}

impl From<Sender<Direction>> for MoveCommandIssuer {
    fn from(command_sender: Sender<Direction>) -> Self {
        Self { command_sender }
    }
}

#[async_trait]
impl OrderMove for MoveCommandIssuer {
    #[tracing::instrument]
    async fn issue_move(&self, direction: Direction) -> Result<(), OrderError> {
        trace!("Issueing new move: {:?}", direction);
        self.command_sender
            .send(direction)
            .await?;

        Ok(())
    }
}

pub fn new_game(fps: f32) -> (Game, MoveCommandIssuer) {
    let (command_sender, command_recv) = mpsc::channel(MOVE_COMMAND_CHANNEL_SIZE);
    let game = Game::new(command_recv.into(), fps);
    let order_move = MoveCommandIssuer::from(command_sender);

    (game, order_move)
}

fn create_game_action_interval(spf: f32) -> Interval {
    let start = Instant::now() + Duration::from_secs(START_DELAY_IN_SECS);
    let mut interval = interval_at(start, Duration::from_secs_f32(spf));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    interval
}


#[tracing::instrument(skip(rng))]
fn pick_move_direction_based_on_probabilities(
    issued_commands: &mut HashMap<Direction, u32>,
    mut rng: &mut impl Rng,
) -> Option<Direction> {
    let (directions, weights): (Vec<Direction>, Vec<u32>) = issued_commands.drain().unzip();
    let dist = WeightedIndex::new(&weights).ok()?;

    let picked_direction = directions[dist.sample(&mut rng)];
    debug!("Picked direction to move: {:?}", picked_direction);

    Some(picked_direction)
}

#[derive(Debug)]
pub struct Game {
    score: u32,
    snake: Snake,
    pub board: Arc<RwLock<Board>>,
    fps: f32,
    move_command_manager_recv: MoveCommandReceiver,
}

impl Game {
    pub async fn start(&mut self) {
        let mut interval = create_game_action_interval(self.convert_fps_to_spf());
        let mut rng = thread_rng();

        let mut direction_command_counters: HashMap<Direction, u32> = HashMap::with_capacity(3);

        loop {
            self.next_frame();
            tokio::select! {
                _ = interval.tick() => {
                    let direction = pick_move_direction_based_on_probabilities(&mut direction_command_counters, &mut rng);

                    match self.snake.make_move(direction) {
                        Err(SnakeError::BitOffHisTail) => {
                            info!("The player bit off his tails, ended up scoring: {}", self.score);
                            break;
                        },
                        Ok(_) => {},
                        Err(SnakeError::BodyIsEmpty) => { // It won't get here since, there is no chance
                                                          // that the body will be emptied
                            error!("Technically this branch is impossible");
                        },
                    }
                }
                _ = self.move_command_manager_recv.wait_for_command_and_act(&mut direction_command_counters, &self.snake.get_current_direction()) => { }
            }
        }
    }

    fn next_frame(&mut self) {
        let mut board = self.board.write().unwrap();

        // Override new head cell with snake head symbol
        (*board).change_cell_symbol(self.snake.head().unwrap(), CellSymbol::SnakeHead);

        // Override old head cell with snake body symbol
        (*board).change_cell_symbol(self.snake.second_segment().unwrap(), CellSymbol::Snake);

        // Override old tail cell with board symbol
        if let Some(point) = self.snake.get_orphaned_tail() {
            (*board).change_cell_symbol(point, CellSymbol::Board);
        }
    }

    fn new(move_command_manager_recv: MoveCommandReceiver, fps: f32) -> Self {
        Self {
            move_command_manager_recv,
            fps,
            score: 0,
            snake: Snake::default(),
            board: Arc::new(RwLock::new(Board::default())),
        }
    }

    fn convert_fps_to_spf(&self) -> f32 {
        1.0 / self.fps as f32
    }

}

