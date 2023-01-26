use super::consts::*;
use std::collections::VecDeque;
use std::ops;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl From<Direction> for i16 {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::Up => -1,
            Direction::Down => 1,
            Direction::Right => 1,
            Direction::Left => -1,
        }
    }
}

struct SnakeIncreaseCommand {}

struct Snake {
    body: VecDeque<Point>,
    increase_snake: Option<SnakeIncreaseCommand>,
    head_current_direction: Direction,
}

fn get_center_of_board_coordinates() -> u16 {
    BOARD_SIZE / 2 + (BOARD_SIZE % 2 != 0) as u16 - 1
}

impl Default for Snake {
    fn default() -> Self {
        let center = get_center_of_board_coordinates();

        let body = (0..3)
            .map(|i| Point {
                x: center,
                y: center + i,
            })
            .collect();

        Snake {
            body,
            increase_snake: None,
            head_current_direction: Direction::Up,
        }
    }
}

#[derive(Error, Debug, PartialEq)]
enum SnakeError {

    #[error("Snake collided with its tail")]
    BitOffHisTail,

    #[error("Snake body is empty")]
    BodyIsEmpty,
}

impl Snake {
    fn new() -> Self {
        Self::default()
    }

    fn test_if_bitten_itself(&self, point: &Point) -> Result<(), SnakeError> {
        match self.body.contains(point) {
            false => Ok(()),
            true => Err(SnakeError::BitOffHisTail),
        }
    }

    fn prepare_new_segment(&mut self) -> Result<Point, SnakeError> {
        let new_segment_or_err = match self.increase_snake {
            None => self.body.pop_back(),
            Some(_) => self.body.back().copied(),
        }
        .ok_or(SnakeError::BodyIsEmpty);

        self.increase_snake = None;
        new_segment_or_err
    }

    fn make_move(&mut self, direction: Option<Direction>) -> Result<(), SnakeError> {
        let direction = direction.unwrap_or(self.head_current_direction);
        if direction.opposite() == self.head_current_direction {
            return Ok(());
        }

        let mut new_segment_to_insert = self.prepare_new_segment()?;

        new_segment_to_insert.set_coords(self.head().ok_or(SnakeError::BodyIsEmpty)?.get_coords());
        new_segment_to_insert += direction;

        self.head_current_direction = direction;

        let result_if_bite = self.test_if_bitten_itself(&new_segment_to_insert);
        self.body.push_front(new_segment_to_insert);

        result_if_bite
    }

    fn head(&self) -> Option<&Point> {
        self.body.front()
    }

    fn size(&self) -> usize {
        self.body.len()
    }

    fn increase_snake_command(&mut self) {
        self.increase_snake = Some(SnakeIncreaseCommand {})
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Point {
    x: u16,
    y: u16,
}

fn add_with_respect_to_bounds(coordinate: u16, move_with_dir: Direction) -> u16 {
    // TODO change it so it's cleaner
    let coordinate_change: i16 = i16::from(move_with_dir);
    if coordinate_change == -1 {
        coordinate.checked_sub(1).unwrap_or(BOARD_SIZE - 1)
    } else if coordinate + 1 >= BOARD_SIZE {
        0
    } else {
        coordinate + coordinate_change as u16
    }
}

impl Point {
    fn set_coords(&mut self, (x, y): (u16, u16)) {
        self.x = x;
        self.y = y;
    }

    fn get_coords(&self) -> (u16, u16) {
        (self.x, self.y)
    }
}

impl ops::AddAssign<Direction> for Point {
    fn add_assign(&mut self, rhs: Direction) {
        let coordinate_ref = match rhs {
            Direction::Up | Direction::Down => &mut self.y,
            Direction::Left | Direction::Right => &mut self.x,
        };

        *coordinate_ref = add_with_respect_to_bounds(*coordinate_ref, rhs);
    }
}

#[cfg(test)]
mod tests {
    use crate::game::snake::{SnakeError, BOARD_SIZE};

    use super::{get_center_of_board_coordinates, Direction, Point, Snake};

    #[test]
    fn test_point_add_assign_increase_y_in_bounds() {
        let direction = Direction::Down;

        let mut point = Point { x: 0, y: 6 };
        point += direction;

        assert_eq!(point.y, 7);
        assert_eq!(point.x, 0);
    }

    #[test]
    fn test_point_add_assign_increase_x_in_bounds() {
        let direction = Direction::Left;

        let mut point = Point { x: 5, y: 0 };
        point += direction;

        assert_eq!(point.y, 0);
        assert_eq!(point.x, 4);
    }

    #[test]
    fn test_point_add_assign_increase_out_of_lower_bound() {
        let direction = Direction::Left;

        let mut point = Point { x: 0, y: 0 };
        point += direction;

        assert_eq!(point.x, BOARD_SIZE as u16 - 1);
    }

    #[test]
    fn test_point_add_assign_increase_out_of_upper_bound() {
        let direction = Direction::Down;

        let mut point = Point {
            x: 0,
            y: BOARD_SIZE as u16 - 1,
        };
        point += direction;

        assert_eq!(point.y, 0);
    }

    #[test]
    fn test_snake_making_moves() {
        let moves = [Direction::Right, Direction::Down, Direction::Up];
        let expected = [Ok(()), Ok(()), Ok(())];
        let mut snake = Snake::new();

        moves
            .into_iter()
            .map(|direction| snake.make_move(Some(direction)))
            .zip(expected)
            .for_each(|(opt, expected)| assert_eq!(opt, expected));
    }

    #[test]
    fn test_snake_moving_without_passed_direction() {
        let mut snake = Snake::new();
        let move_result = snake.make_move(None);
        let center = get_center_of_board_coordinates();

        let expected_point = Point { x: center, y: center - 1 };

        assert_eq!(move_result.err(), None);
        assert_eq!(*snake.head().unwrap(), expected_point);
    }

    #[test]
    fn test_snake_head_positions_while_moving() {
        let mut snake = Snake::new();
        let center = get_center_of_board_coordinates();
        let mut point = Point { x: center, y:  center};

        assert_eq!(*snake.head().unwrap(), point);

        let _ = snake.make_move(Some(Direction::Right));
        point.x = 10;
        assert_eq!(*snake.head().unwrap(), point);

        _ = snake.make_move(Some(Direction::Down));
        point.y = 10;
        assert_eq!(*snake.head().unwrap(), point);

        _ = snake.make_move(Some(Direction::Up));
        assert_eq!(*snake.head().unwrap(), point);
    }

    #[test]
    fn test_snake_head_positions_while_moved_outside_of_bounds() {
        let mut snake = Snake::new();
        let center = get_center_of_board_coordinates();

        (center..BOARD_SIZE).chain(0..=center)
            .map(|i| Point { x: i, y: center })
            .for_each(|point| {
                assert_eq!(*snake.head().unwrap(), point);
                let _ = snake.make_move(Some(Direction::Right));
            })
    }

    #[test]
    fn test_if_snake_bites_itself_results_in_error() {
        let mut snake = Snake::new();

        // Increase size of the size, so its length is 5. It allows snake to bite itself
        snake.increase_snake_command();

        let mut result = snake.make_move(Some(Direction::Right));
        assert_eq!(result.err(), None);
        snake.increase_snake_command();

        result = snake.make_move(Some(Direction::Down));
        assert_eq!(result.err(), None);

        result = snake.make_move(Some(Direction::Left));
        assert_eq!(result, Err(SnakeError::BitOffHisTail));
    }

    #[test]
    fn test_if_snake_size_increasing_command_adds_new_segments() {
        let mut snake = Snake::new();

        assert_eq!(snake.size(), 3);

        snake.increase_snake_command();
        _ = snake.make_move(None);
        assert_eq!(snake.size(), 4);


        snake.increase_snake_command();
        _ = snake.make_move(None);
        assert_eq!(snake.size(), 5);

    }

}
