use super::board::{generate_points_pool, Board, CellSymbol};
use super::consts::*;
use super::fruit::Fruit;
use super::point::{Direction, Point};
use super::snake::{Snake, SnakeError};
use crate::server;
use std::collections::HashMap;
use std::sync::RwLock;

use tokio::signal;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::TryRecvError;

use super::commands::{MoveCommandIssuer, MoveCommandReceiver};

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::time::{interval_at, Duration, Instant, Interval, MissedTickBehavior};
use tracing::{debug, error, info, warn};

const MOVE_COMMAND_CHANNEL_SIZE: usize = 1000;

pub async fn game_loop(
    terminal_signal_tx: broadcast::Sender<()>,
    command_receiver: MoveCommandReceiver,
    order_move: Arc<RwLock<MoveCommandIssuer>>,
    board: Arc<RwLock<Board>>,
    fps: f32,
) {
    let mut game = Game::new(command_receiver, Arc::clone(&board), fps);
    loop {
        let rx = terminal_signal_tx.subscribe();
        tokio::select! {
            _ = game.start(rx) => {
                let (command_sender, command_recv) = mpsc::channel(MOVE_COMMAND_CHANNEL_SIZE);

                let command_receiver = command_recv.into();
                order_move.write().unwrap().set_issuer(command_sender);
                *board.write().unwrap() = Board::default();

                game = Game::new(command_receiver, Arc::clone(&board), fps);
            }
            _ = signal::ctrl_c() => {
                if let Err(err) = terminal_signal_tx.send(()) {
                    warn!("Termination signal {}", err)
                };
                break;
            }
        }
    }
}

pub async fn new_game(fps: f32) {
    // Movement command channels
    let (command_sender, command_recv) = mpsc::channel(MOVE_COMMAND_CHANNEL_SIZE);

    // Arc<RwLock<Board>> since these variables/objects are read from other thread
    let board = Arc::new(RwLock::new(Board::default()));
    let order_move = Arc::new(RwLock::new(MoveCommandIssuer::from(command_sender)));

    let server_running = server::run(Arc::clone(&order_move), Arc::clone(&board));

    // Termination signal channel
    let (terminal_signal_tx, _) = broadcast::channel(1);
    let tx_for_server = terminal_signal_tx.clone();

    // Spawn thread with game loop
    let game_loop_task = tokio::spawn(async move {
        game_loop(
            terminal_signal_tx,
            command_recv.into(),
            order_move,
            board,
            fps,
        )
        .await
    });

    // End program when one of its components is done
    tokio::select! {
        _ = server_running => {
            if let Err(err) = tx_for_server.send(()) {
                error!("{}", err)
            };
        }
        _ = game_loop_task => {
            warn!("Game loop finished");
        }
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
    board: Arc<RwLock<Board>>,
    fps: f32,
    move_command_manager_recv: MoveCommandReceiver,
}

impl Game {
    pub async fn start(&mut self, mut shutdown_signal_recv: broadcast::Receiver<()>) {
        let mut interval = create_game_action_interval(self.convert_fps_to_spf());
        let mut direction_command_counters: HashMap<Direction, u32> = HashMap::with_capacity(3);
        let points_pool: Vec<Point> = generate_points_pool();

        loop {
            match shutdown_signal_recv.try_recv() {
                Ok(_) | Err(TryRecvError::Closed) => break,
                _ => {}
            };

            self.next_frame();

            self.control_fruits(&points_pool);
            tokio::select! {
                _ = interval.tick() => {
                    if self.control_movement(&mut direction_command_counters).is_none() {
                        break
                    }
                }
                _ = self.move_command_manager_recv.wait_for_command_and_act(&mut direction_command_counters, self.snake.get_current_direction()) => { }
            }
            self.check_if_snake_ate_fruit();
        }
    }

    fn control_movement(
        &mut self,
        direction_command_counters: &mut HashMap<Direction, u32>,
    ) -> Option<()> {
        let direction = pick_move_direction_based_on_probabilities(
            direction_command_counters,
            &mut thread_rng(),
        );

        match self.snake.make_move(direction) {
            Err(SnakeError::BitOffHisTail) => {
                info!(
                    "The player bit off his tails, ended up scoring: {}",
                    self.score
                );
                None
            }
            Ok(_) => Some(()),
            Err(SnakeError::BodyIsEmpty) => {
                // It won't get here since, there is no chance
                // that the body will be empty
                unreachable!()
            }
        }
    }

    fn control_fruits(&mut self, points_pool: &[Point]) {
        if self.fruits.len() < 5 {
            let occupied_snake = self.snake.get_occupied_points();

            // Get possible points to place a new fruit
            let next_frame_filtered_out_cells = points_pool
                .iter()
                .filter(|&p| !occupied_snake.contains(p))
                .collect();

            // Try spawning a new fruit
            if let Some(fruit) =
                Fruit::try_spawn_at_random_place(&next_frame_filtered_out_cells, self.fruits.len())
            {
                self.fruits.push(fruit);
            };
        }
    }

    fn check_if_snake_ate_fruit(&mut self) {
        if remove_eaten_fruits(&mut self.fruits, self.snake.head().unwrap()) {
            self.snake.increase_snake_command();
            self.score += 1;
        }
    }

    fn next_frame(&mut self) {
        let mut board = self.board.write().unwrap();

        self.fruits
            .iter()
            .for_each(|f| (*board).change_cell_symbol(&f.point, CellSymbol::Fruit));

        // Override new head cell with snake head symbol
        (*board).change_cell_symbol(self.snake.head().unwrap(), CellSymbol::SnakeHead);

        // Override old head cell with snake body symbol
        (*board).change_cell_symbol(self.snake.second_segment().unwrap(), CellSymbol::Snake);

        // Override old tail cell with board symbol
        if let Some(point) = self.snake.get_orphaned_tail() {
            (*board).change_cell_symbol(point, CellSymbol::Board);
        }
    }

    fn new(
        move_command_manager_recv: MoveCommandReceiver,
        board: Arc<RwLock<Board>>,
        fps: f32,
    ) -> Self {
        Self {
            move_command_manager_recv,
            fps,
            score: 0,
            snake: Snake::default(),
            fruits: vec![],
            board,
        }
    }

    fn convert_fps_to_spf(&self) -> f32 {
        1.0 / self.fps
    }
}

fn remove_eaten_fruits(fruits: &mut Vec<Fruit>, actual_point: &Point) -> bool {
    let mut removed = false;
    fruits.retain(|fruit| {
        if fruit.point == *actual_point {
            removed = true;
            false
        } else {
            true
        }
    });
    removed
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_removing_fruits_on_eat() {
        let mut fruits = vec![Fruit {
            point: Point::new(2, 5),
        }];
        let snake_head = Point::new(2, 5);

        let removed = remove_eaten_fruits(&mut fruits, &snake_head);

        assert!(removed);
        assert_eq!(fruits.len(), 0)
    }

    #[test]
    fn test_not_removing_fruits_on_move_without_eating() {
        let mut fruits = vec![Fruit {
            point: Point::new(2, 5),
        }];
        let snake_head = Point::new(3, 5);

        let removed = remove_eaten_fruits(&mut fruits, &snake_head);

        assert!(!removed);
        assert_eq!(fruits.len(), 1)
    }
}
