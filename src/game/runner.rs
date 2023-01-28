use tokio::sync::broadcast;
use tokio::sync::broadcast::error::TryRecvError;
use crate::server;
use super::fruit::Fruit;
use std::thread;
use super::board::{Board, CellSymbol, generate_points_pool};
use super::consts::*;
use super::point::{Direction, Point};
use super::snake::{Snake, SnakeError};
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::task;
use tokio::signal;


use rand::distributions::WeightedIndex;
use rand::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::{interval_at, Duration, Instant, Interval, MissedTickBehavior};
use tracing::{debug, error, info, trace, warn};
use super::movement::OrderMove;
use super::commands::{MoveCommandReceiver, MoveCommandIssuer};

const MOVE_COMMAND_CHANNEL_SIZE: usize = 1000;

pub async fn new_game(fps: f32) {
    let (command_sender, command_recv) = mpsc::channel(MOVE_COMMAND_CHANNEL_SIZE);
    let command_receiver = command_recv.into();

    let board = Arc::new(RwLock::new(Board::default()));
    let order_move = Arc::new(RwLock::new(MoveCommandIssuer::from(command_sender)));

    let mut game = Game::new(command_receiver, Arc::clone(&board), fps);

    let (terminal_signal_tx, _) = broadcast::channel(1);
    let tx_for_server =  terminal_signal_tx.clone();

    let server_running = server::run(Arc::clone(&order_move), Arc::clone(&board));

    let game_loop = tokio::spawn(async move {
        loop {
            let rx = terminal_signal_tx.subscribe();
            tokio::select!{
                _ = game.start(rx) => {
                    let (command_sender, command_recv) = mpsc::channel(MOVE_COMMAND_CHANNEL_SIZE);

                    let command_receiver = command_recv.into();
                    order_move.write().unwrap().set_issuer(command_sender);
                    *board.write().unwrap() = Board::default();

                    game = Game::new(command_receiver, Arc::clone(&board), fps);
                }
                _ = signal::ctrl_c() => {
                    terminal_signal_tx.send(());
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = server_running => {
            tx_for_server.send(());
        }
        _ = game_loop => {}
    }
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
    fruits: Vec<Fruit>,
    pub board: Arc<RwLock<Board>>,
    fps: f32,
    move_command_manager_recv: MoveCommandReceiver,
}



impl Game {
    pub async fn start(&mut self, mut shutdown_signal_recv: broadcast::Receiver<()>) {
        let mut interval = create_game_action_interval(self.convert_fps_to_spf());
        let mut direction_command_counters: HashMap<Direction, u32> = HashMap::with_capacity(3);
        let filtered_out_occupied_points: Vec<Point> = generate_points_pool();

        loop {
            match shutdown_signal_recv.try_recv() {
                Ok(_) | Err(TryRecvError::Closed) => break,
                _ => {},
            };

            self.next_frame();

            filtered_out_occupied_points.filter();
            if let Some(fruit) = Fruit::try_spawn_at_random_place(&filtered_out_occupied_points) {
                self.fruits.push(fruit);
            };

            tokio::select! {
                _ = interval.tick() => {
                    let direction = pick_move_direction_based_on_probabilities(&mut direction_command_counters, &mut thread_rng());

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

    fn new(move_command_manager_recv: MoveCommandReceiver, board: Arc<RwLock<Board>>, fps: f32) -> Self {
        Self {
            move_command_manager_recv,
            fps,
            score: 0,
            snake: Snake::default(),
            board
        }
    }

    fn convert_fps_to_spf(&self) -> f32 {
        1.0 / self.fps as f32
    }

}

