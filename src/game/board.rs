use super::consts::*;
use super::point::Point;

use std::fmt::{Display, Write};
const CANVAS_SIZE_X: usize = BOARD_SIZE_X as usize + 2;
const CANVAS_SIZE_Y: usize = BOARD_SIZE_Y as usize + 2;

const SE: char = '┌';
const SW: char = '┐';
const NW: char = '┘';
const NE: char = '└';

const NS: char = '│';
const EW: char = '─';

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
    fn to_char(self) -> char {
        match self {
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

pub fn get_center_of_board_coordinates() -> Point {
    let y = BOARD_SIZE_Y / 2 + (BOARD_SIZE_Y % 2 != 0) as u16 - 1;
    let x = BOARD_SIZE_X / 2 + (BOARD_SIZE_X % 2 != 0) as u16 - 1;

    Point::new(y, x)
}

pub fn generate_points_pool() -> Vec<Point> {
    (0..BOARD_SIZE_Y)
        .into_iter()
        .flat_map(|y| {
            let copy_y = y;
            (0..BOARD_SIZE_X).map(move |x| Point { x, y: copy_y })
        })
        .collect()
}

type Canvas = [[CellSymbol; CANVAS_SIZE_X]; CANVAS_SIZE_Y];

#[derive(Debug)]
pub struct Board {
    // This could be a RwLock,
    // so it would avoid reading partial updates
    canvas: Canvas,
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
        let mut canvas = [[CellSymbol::Board; CANVAS_SIZE_X]; CANVAS_SIZE_Y];

        for i in 1..(CANVAS_SIZE_Y - 1) {
            // set '|' for vertical walls
            canvas[i][0] = CellSymbol::Wall(Wall::NS);
            canvas[i][CANVAS_SIZE_X - 1] = CellSymbol::Wall(Wall::NS);
        }

        for i in 1..(CANVAS_SIZE_X - 1) {
            // set '-' for horizontal walls
            canvas[0][i] = CellSymbol::Wall(Wall::EW);
            canvas[CANVAS_SIZE_Y - 1][i] = CellSymbol::Wall(Wall::EW);
        }

        // set proper symbol for corner cells
        canvas[0][0] = CellSymbol::Junction(Junction::SE);
        canvas[CANVAS_SIZE_Y - 1][CANVAS_SIZE_X - 1] = CellSymbol::Junction(Junction::NW);
        canvas[0][CANVAS_SIZE_X - 1] = CellSymbol::Junction(Junction::SW);
        canvas[CANVAS_SIZE_Y - 1][0] = CellSymbol::Junction(Junction::NE);

        Board { canvas }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_default_cavas() {
        let raw_canvas = r#"┌────────────────────────────────────────┐
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
│                                        │
└────────────────────────────────────────┘
"#;

        let board = Board::default();
        let mut generated_board_str = String::new();
        board.get_board(&mut generated_board_str).unwrap();

        assert_eq!(raw_canvas, generated_board_str.as_str());
    }
}
