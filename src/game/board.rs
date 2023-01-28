use super::{consts::*, point};
use super::point::Point;

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

trait ChangeableCell {
    fn change_to_new_symbol(&self, symbol: CellSymbol);
}

type Canvas = [[CellSymbol; CANVAS_SIZE]; CANVAS_SIZE];

#[derive(Debug)]
pub struct Board {

    // This could be a RwLock,
    // so it would avoid reading partial updates
    canvas: Canvas
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
        let mut data = [[CellSymbol::Board; CANVAS_SIZE]; CANVAS_SIZE];

        for i in 1..(CANVAS_SIZE - 1) {
            // set '|' for vertical walls
            data[i][0] = CellSymbol::Wall(Wall::NS);
            data[i][CANVAS_SIZE - 1] = CellSymbol::Wall(Wall::NS);

            // set '-' for horizontal walls
            data[0][i] = CellSymbol::Wall(Wall::EW);
            data[CANVAS_SIZE - 1][i] = CellSymbol::Wall(Wall::EW);
        }

        // set proper symbol for corner cells
        data[0][0] = CellSymbol::Junction(Junction::SE);
        data[CANVAS_SIZE - 1][CANVAS_SIZE - 1] = CellSymbol::Junction(Junction::NW);
        data[0][CANVAS_SIZE - 1] = CellSymbol::Junction(Junction::SW);
        data[CANVAS_SIZE - 1][0] = CellSymbol::Junction(Junction::NE);

        Board { canvas: data }
    }
}
