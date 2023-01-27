use super::board::Board;
use super::consts::*;
use super::point::Direction;
use super::snake::{Snake, SnakeError};
use std::collections::HashMap;


use rand::distributions::WeightedIndex;
use rand::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::{interval_at, Duration, Instant, Interval, MissedTickBehavior};
use tracing::{debug, error, info, trace, warn};

const MOVE_COMMAND_CHANNEL_SIZE: usize = 1000;

#[derive(Debug)]
struct MoveCommandManager {
    command_rx: Receiver<Direction>,
    command_sender: Sender<Direction>,
}

impl Default for MoveCommandManager {
    fn default() -> Self {
        let (command_sender, command_rx) = mpsc::channel(MOVE_COMMAND_CHANNEL_SIZE);
        MoveCommandManager {
            command_rx,
            command_sender,
        }
    }
}

#[tracing::instrument]
pub async fn start_game(fps: f32) {
    let mut game = Game {
        fps,
        ..Default::default()
    };

    game.start().await;
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
    trace!("Picked direction: {:?}", picked_direction);

    Some(picked_direction)
}

#[derive(Default, Debug)]
struct Game {
    score: u32,
    snake: Snake,
    board: Board,
    fps: f32,
    move_command_manager: MoveCommandManager,
}

impl Game {
    pub async fn start(&mut self) {
        let mut interval = create_game_action_interval(self.convert_fps_to_spf());
        let mut rng = thread_rng();

        let mut direction_command_counters: HashMap<Direction, u32> = HashMap::with_capacity(3);

        trace!("Whats happening");
        loop {
            trace!("Whats happening");
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
                command = self.move_command_manager.command_rx.recv() => {
                    match command {
                        Some(c) => {
                            direction_command_counters.entry(c).and_modify(|counter| *counter +=  1).or_insert(1);
                            debug!("Received a move command from user {:?}", command);
                        }
                        None => {
                            warn!("Received a move command although it was empty");
                        }
                    };
                }
            }
        }
    }

    fn convert_fps_to_spf(&self) -> f32 {
        1.0 / self.fps as f32
    }
}
