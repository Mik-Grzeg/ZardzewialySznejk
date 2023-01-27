use std::{fmt::{Display, Write}, io::IoSlice};
use lazy_static::lazy_static;
use super::consts::*;
const BOARD_USIZE: usize = BOARD_SIZE as usize;

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
enum Junction {
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
enum Wall {
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
enum CellSymbol {
    Board,
    Snake,
    SnakeHead,
    Fruit,
    Wall(Wall),
    Junction(Junction)
}

impl CellSymbol {
    fn to_char(&self) -> char {
        match *self {
            CellSymbol::Board => ' ',
            CellSymbol::Snake => '#',
            CellSymbol::SnakeHead => '@',
            CellSymbol::Fruit => 'O',
            CellSymbol::Wall(wall) => wall.into(),
            CellSymbol::Junction(junction) => junction.into()
        }
    }
}

impl Display for CellSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = self.to_char();
        write!(f, "{symbol}")
    }
}

type Canvas = [[CellSymbol; BOARD_USIZE]; BOARD_USIZE];


#[derive(Debug)]
pub struct Board {
    canvas: Canvas,
    // cached_printable_canvas: [u8; BOARD_USIZE * BOARD_USIZE],
}

impl Board {
    pub fn get_board(&self, wr: &mut impl Write) -> Result<(), std::fmt::Error>{
        for row in self.canvas {
            for cell in row {
                wr.write_char(cell.to_char())?;
            }
            wr.write_char('\n')?;
        }

        Ok(())
    }

}

impl Default for Board {
    fn default() -> Board {
        let mut data = [[CellSymbol::Board; BOARD_USIZE]; BOARD_USIZE];

        for i in 1..(BOARD_USIZE-1) {

            // set '|' for vertical walls
            data[i][0] = CellSymbol::Wall(Wall::NS);
            data[i][BOARD_USIZE - 1] = CellSymbol::Wall(Wall::NS);

            // set '-' for horizontal walls
            data[0][i] = CellSymbol::Wall(Wall::EW);
            data[BOARD_USIZE - 1][i] = CellSymbol::Wall(Wall::EW);
        }

        // set proper symbol for corner cells
        data[0][0] = CellSymbol::Junction(Junction::SE);
        data[BOARD_USIZE-1][BOARD_USIZE-1] = CellSymbol::Junction(Junction::NW);
        data[0][BOARD_USIZE-1] = CellSymbol::Junction(Junction::SW);
        data[BOARD_USIZE-1][0] = CellSymbol::Junction(Junction::NE);

        Board {
            canvas: data
        }
    }
}
