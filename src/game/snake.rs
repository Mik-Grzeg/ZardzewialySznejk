use super::board::get_center_of_board_coordinates;

use super::point::*;
use std::collections::VecDeque;

use thiserror::Error;
use tracing::trace;

#[derive(Debug)]
struct SnakeIncreaseCommand {}

#[derive(Debug)]
pub struct Snake {
    body: VecDeque<Point>,
    increase_snake: Option<SnakeIncreaseCommand>,
    head_current_direction: Direction,
    orphaned_tail: Point,
}

impl Default for Snake {
    fn default() -> Self {
        let center = get_center_of_board_coordinates();

        let body: VecDeque<Point> = (0..3).map(|i| Point::new(center.y + i, center.x)).collect();

        Snake {
            orphaned_tail: body.back().copied().unwrap(),
            body,
            increase_snake: None,
            head_current_direction: Direction::Up,
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SnakeError {
    #[error("Snake collided with its tail")]
    BitOffHisTail,

    #[error("Snake body is empty")]
    BodyIsEmpty,
}

impl Snake {
    fn check_if_bitten_itself(&self, point: &Point) -> Result<(), SnakeError> {
        match self.body.contains(point) {
            false => Ok(()),
            true => Err(SnakeError::BitOffHisTail),
        }
    }

    fn prepare_new_segment(&mut self) -> Result<Point, SnakeError> {
        // If there was no command to increase snake, remove last segment and return it
        // otherwise copy the last segment and return it
        let new_segment_or_err = match self.increase_snake {
            None => {
                let old_tail = self.body.pop_back();
                self.orphaned_tail = old_tail.unwrap();
                old_tail
            }
            Some(_) => self.body.back().copied(),
        }
        .ok_or(SnakeError::BodyIsEmpty);

        self.increase_snake = None;
        new_segment_or_err
    }

    #[tracing::instrument(skip(self))]
    pub fn make_move(&mut self, direction: Option<Direction>) -> Result<(), SnakeError> {
        let direction = direction.unwrap_or(self.head_current_direction);
        if direction.opposite() == self.head_current_direction {
            return Ok(());
        }

        let mut new_segment_to_insert = self.prepare_new_segment()?;

        // Set new head coordinations
        new_segment_to_insert.set_coords(self.head().ok_or(SnakeError::BodyIsEmpty)?.get_coords());
        // Move in specific direction
        new_segment_to_insert += direction;

        self.head_current_direction = direction;

        let result_if_bite = self.check_if_bitten_itself(&new_segment_to_insert);
        self.body.push_front(new_segment_to_insert);

        trace!("Head moved to {:?}", new_segment_to_insert);

        result_if_bite
    }

    pub fn head(&self) -> Option<&Point> {
        self.body.front()
    }

    pub fn second_segment(&self) -> Option<&Point> {
        self.body.get(1)
    }

    pub fn get_orphaned_tail(&self) -> Option<&Point> {
        match self.increase_snake {
            None => Some(&self.orphaned_tail),
            Some(_) => None,
        }
    }

    pub fn get_occupied_points(&self) -> &VecDeque<Point> {
        &self.body
    }

    pub fn increase_snake_command(&mut self) {
        self.increase_snake = Some(SnakeIncreaseCommand {})
    }

    pub fn get_current_direction(&self) -> &Direction {
        &self.head_current_direction
    }

    #[cfg(test)]
    fn size(&self) -> usize {
        self.body.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{get_center_of_board_coordinates, Direction, Point, Snake};
    use crate::game::consts::*;
    use crate::game::snake::SnakeError;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_snake_making_moves() {
        let moves = [Direction::Right, Direction::Down, Direction::Up];
        let expected = [Ok(()), Ok(()), Ok(())];
        let mut snake = Snake::default();

        moves
            .into_iter()
            .map(|direction| snake.make_move(Some(direction)))
            .zip(expected)
            .for_each(|(opt, expected)| assert_eq!(opt, expected));
    }

    #[test]
    fn test_snake_moving_without_passed_direction() {
        let mut snake = Snake::default();
        let move_result = snake.make_move(None);
        let center = get_center_of_board_coordinates();

        let expected_point = Point::new(center.y - 1, center.x);

        assert_eq!(move_result.err(), None);
        assert_eq!(*snake.head().unwrap(), expected_point);
    }

    #[test]
    fn test_snake_head_positions_while_moving() {
        let mut snake = Snake::default();
        let center = get_center_of_board_coordinates();
        let mut point = Point::new(center.y, center.x);

        assert_eq!(*snake.head().unwrap(), point);

        let _ = snake.make_move(Some(Direction::Right));
        point.x = center.x + 1;
        assert_eq!(*snake.head().unwrap(), point);

        _ = snake.make_move(Some(Direction::Down));
        point.y = center.y + 1;
        assert_eq!(*snake.head().unwrap(), point);

        _ = snake.make_move(Some(Direction::Up));
        assert_eq!(*snake.head().unwrap(), point);
    }

    #[test]
    fn test_snake_head_positions_while_moved_outside_of_bounds() {
        let mut snake = Snake::default();
        let center = get_center_of_board_coordinates();

        (center.x..BOARD_SIZE_X)
            .chain(0..=center.x)
            .map(|i| Point::new(center.y, i))
            .for_each(|point| {
                assert_eq!(*snake.head().unwrap(), point);
                let _ = snake.make_move(Some(Direction::Right));
            })
    }

    #[test]
    fn test_if_snake_bites_itself_results_in_error() {
        let mut snake = Snake::default();

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
        let mut snake = Snake::default();

        assert_eq!(snake.size(), 3);

        snake.increase_snake_command();
        _ = snake.make_move(None);
        assert_eq!(snake.size(), 4);

        snake.increase_snake_command();
        _ = snake.make_move(None);
        assert_eq!(snake.size(), 5);
    }
}
