use super::consts::*;
use std::ops;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Error)]
pub enum DirectionError {
    #[error("There are 4 possible directions: ['left', 'right', 'up', 'down']. `{0}` does not match any of them")]
    ConversionFromStringError(String),
}

impl TryFrom<String> for Direction {
    type Error = DirectionError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            _ => Err(DirectionError::ConversionFromStringError(value)),
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

fn add_with_respect_to_bounds(coordinate: u16, move_with_dir: Direction, bound: u16) -> u16 {
    let coordinate_change: i16 = i16::from(move_with_dir);
    if coordinate_change == -1 {
        coordinate.checked_sub(1).unwrap_or(bound - 1)
    } else if coordinate + 1 >= bound {
        0
    } else {
        coordinate + coordinate_change as u16
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn new(y: u16, x: u16) -> Self {
        Self { y, x }
    }

    pub fn set_coords(&mut self, (y, x): (u16, u16)) {
        self.x = x;
        self.y = y;
    }

    pub fn get_coords(&self) -> (u16, u16) {
        (self.y, self.x)
    }
}

impl ops::AddAssign<Direction> for Point {
    fn add_assign(&mut self, rhs: Direction) {
        match rhs {
            Direction::Up | Direction::Down => {
                self.y = add_with_respect_to_bounds(self.y, rhs, BOARD_SIZE_Y);
            }
            Direction::Left | Direction::Right => {
                self.x = add_with_respect_to_bounds(self.x, rhs, BOARD_SIZE_X);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::consts::*;
    use super::{Direction, Point};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_point_add_assign_increase_y_in_bounds() {
        let direction = Direction::Down;

        let mut point = Point::new(6, 0);
        point += direction;

        assert_eq!(point.y, 7);
        assert_eq!(point.x, 0);
    }

    #[test]
    fn test_point_add_assign_increase_x_in_bounds() {
        let direction = Direction::Left;

        let mut point = Point::new(0, 5);
        point += direction;

        assert_eq!(point.y, 0);
        assert_eq!(point.x, 4);
    }

    #[test]
    fn test_point_add_assign_increase_out_of_lower_bound() {
        let direction = Direction::Left;

        let mut point = Point::new(0, 0);
        point += direction;

        assert_eq!(point.x, BOARD_SIZE_X - 1);
    }

    #[test]
    fn test_point_add_assign_increase_out_of_upper_bound() {
        let direction = Direction::Down;

        let mut point = Point::new(BOARD_SIZE_Y - 1, 0);
        point += direction;

        assert_eq!(point.y, 0);
    }
}
