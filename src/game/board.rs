use super::{consts::*, point};
use super::point::{Point, State};
use rand::seq::{SliceRandom, IteratorRandom};

use std::{
    fmt::{Display, Write},
};
const CANVAS_SIZE: usize = BOARD_SIZE as usize + 2 ;

const SE: char = '┌';
const SW: char = '┐';
const NW: char = '┘';
const NE: char = '└';

const NS: char = '│';
const EW: char = '─';

const EWS: char = '┬';
const NES: char = '├';
const NWS: char = '┤';
const NEW: char = '┴';

const NEWS: char = '┼';

#[derive(Debug, Clone, Copy)]
pub enum Junction {
    NE,
    NW,
    SE,
    SW,
}

impl From<Junction> for char {
    fn from(j: Junction) -> Self {
        match j {
            Junction::NE => NE,
            Junction::NW => NW,
            Junction::SE => SE,
            Junction::SW => SW,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Wall {
    NS,
    EW,
}

impl From<Wall> for char {
    fn from(w: Wall) -> Self {
        match w {
            Wall::NS => NS,
            Wall::EW => EW,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CellSymbol {
    Board,
    Snake,
    SnakeHead,
    Fruit,
    Wall(Wall),
    Junction(Junction),
}

impl CellSymbol {
    fn to_char(&self) -> char {
        match *self {
            CellSymbol::Board => ' ',
            CellSymbol::Snake => '#',
            CellSymbol::SnakeHead => '@',
            CellSymbol::Fruit => 'O',
            CellSymbol::Wall(wall) => wall.into(),
            CellSymbol::Junction(junction) => junction.into(),
        }
    }
}

impl Display for CellSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = self.to_char();
        write!(f, "{symbol}")
    }
}

// trait ChangeableCell {
//     fn change_to_new_symbol(&self, symbol: CellSymbol);
// }

// #[derive(Debug)]
// pub struct PointPool {
//     free: Vec<Point>,
//     occupied: Vec<Point>,
// }



// impl PointPool {
//     pub fn take_out_random_free_point(&mut self) -> Point {
//         let i = (0..self.free.len()).choose(&mut rand::thread_rng()).unwrap(); // handle unwrap
//         self.free.swap_remove(i)
//     }

//     pub fn add_new_free_point(&mut self, point: Point) {
//         self.free.push(point);
//     }

//     pub fn take_out_specific_free_point(&mut self, (y, x): (u16, u16)) -> Option<Point> {
//         let index = self.free.iter().position(|value| value.x == x && value.y == y)?;
//         Some(self.free.swap_remove(index))
//     }
// }

// impl Default for PointPool {
pub fn generate_points_pool() -> Vec<Point> {
    (0..BOARD_SIZE)
        .into_iter()
        .map(|y| {
            let copy_y = y;
            (0..BOARD_SIZE)
                .map(move |x| {
                    Point { x,  y: copy_y, state: State::Free }
                })
        })
        .flatten()
        .collect()
}

type Canvas = [[CellSymbol; CANVAS_SIZE]; CANVAS_SIZE];

#[derive(Debug)]
pub struct Board {

    // This could be a RwLock,
    // so it would avoid reading partial updates
    canvas: Canvas,
    // point_pool: PointPool,
}

impl Board {
    pub fn get_board(&self, wr: &mut impl Write) -> Result<(), std::fmt::Error> {
        for row in self.canvas {
            for cell in row {
                wr.write_char(cell.to_char())?;
            }
            wr.write_char('\n')?;
        }

        Ok(())
    }

    fn translate_points_to_cavas_points(&self, point: &Point) -> (usize, usize) {
        let (y, x) = point.get_coords();
        (y as usize + 1, x as usize + 1)
    }

    pub fn change_cell_symbol(&mut self, point: &Point, symbol: CellSymbol) {
        let (y, x) = self.translate_points_to_cavas_points(point);
        self.canvas[y][x] = symbol;
    }
}

impl Default for Board {
    fn default() -> Board {
        let mut canvas = [[CellSymbol::Board; CANVAS_SIZE]; CANVAS_SIZE];

        for i in 1..(CANVAS_SIZE - 1) {
            // set '|' for vertical walls
            canvas[i][0] = CellSymbol::Wall(Wall::NS);
            canvas[i][CANVAS_SIZE - 1] = CellSymbol::Wall(Wall::NS);

            // set '-' for horizontal walls
            canvas[0][i] = CellSymbol::Wall(Wall::EW);
            canvas[CANVAS_SIZE - 1][i] = CellSymbol::Wall(Wall::EW);
        }

        // set proper symbol for corner cells
        canvas[0][0] = CellSymbol::Junction(Junction::SE);
        canvas[CANVAS_SIZE - 1][CANVAS_SIZE - 1] = CellSymbol::Junction(Junction::NW);
        canvas[0][CANVAS_SIZE - 1] = CellSymbol::Junction(Junction::SW);
        canvas[CANVAS_SIZE - 1][0] = CellSymbol::Junction(Junction::NE);

        Board { canvas }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};


    #[test]
    fn test_default_cavas() {
        let raw_canvas = r#"┌────────────────────┐
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
│                    │
└────────────────────┘
"#;

        let board = Board::default();
        let mut generated_board_str = String::new();
        board.get_board(&mut generated_board_str).unwrap();

        assert_eq!(raw_canvas, generated_board_str.as_str());
    }
}

