use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use serde::Deserialize;

#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum FaceType {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Mirror,
    Boom,
}

#[derive(Deserialize, Copy, Clone)]
pub enum CellType {
    #[serde(rename = " ")]
    None,
    #[serde(rename = "*")]
    Basic,
    #[serde(rename = "^")]
    SlideUp,
    #[serde(rename = "V")]
    SlideDown,
    #[serde(rename = "<")]
    SlideLeft,
    #[serde(rename = ">")]
    SlideRight,
}

#[derive(Deserialize, Copy, Clone)]
pub struct Face {
    pub symbol: FaceType,
    #[serde(default)]
    pub dx: i8,
    #[serde(default)]
    pub dy: i8,
}

#[derive(Deserialize, Clone)]
pub struct Polyomino {
    #[serde(default)]
    pub x: u8,
    #[serde(default)]
    pub y: u8,
    #[serde(default)]
    pub shield: u8,
    pub faces: Vec<Face>,
}

#[derive(Deserialize, Clone)]
pub struct Level {
    pub polyominoes: Vec<Polyomino>,
    pub hand: Vec<Polyomino>,
    pub cells: Vec<Vec<CellType>>,
}

#[derive(Clone)]
pub struct FaceRef {
    pub polyomino: Rc<Polyomino>,
    pub index: u8,
}

pub struct Board {
    pub width: u8,
    pub height: u8,
    pub cells: Vec<CellType>,
    pub faces: Vec<Option<FaceRef>>,
    pub hand: Vec<Rc<Polyomino>>,
}

impl Board {
    pub fn index_of(&self, x: u8, y: u8) -> Option<usize> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(y as usize * self.width as usize + x as usize)
        }
    }

    pub fn cell_at(&self, x: u8, y: u8) -> Option<CellType> {
        self.index_of(x, y)
            .and_then(|index| Some(self.cells[index]))
            .and_then(|cell| match cell {
                CellType::None => None,
                _ => Some(cell),
            })
    }

    pub fn face_at(&self, x: u8, y: u8) -> Option<FaceType> {
        self.index_of(x, y)
            .and_then(|index| self.faces[index].as_ref())
            .and_then(|face| Some(face.polyomino.faces[face.index as usize].symbol))
    }
}

fn create_board(level: &Level) -> Result<Board, Box<dyn Error>> {
    let raw_width = level.cells.iter().map(Vec::len).max();
    let raw_height = level.cells.len();

    let width = raw_width.unwrap_or_default();
    let height = raw_height;

    if width == 0 || height == 0 {
        return Err("Board grid must have at least one row and column".into());
    }
    if width >= 256 || height >= 256 {
        return Err("Board grid cannot have more than 255 rows or columns".into());
    }

    let resize_row = |row: &mut Vec<CellType>| row.resize(width, CellType::None);
    let mut level_cells = level.cells.clone();
    level_cells.resize(height, vec![]);
    level_cells.iter_mut().for_each(resize_row);
    let cells = level_cells.into_iter().flatten().collect();

    let mut faces = Vec::new();
    faces.resize(width * height, None);
    for polyomino in level.polyominoes.iter() {
        if polyomino.faces.len() >= 256 {
            return Err("Polyomino cannot have more than 255 faces".into());
        }

        let rc = Rc::new(polyomino.clone());
        for (i, face) in polyomino.faces.iter().enumerate() {
            let cx = polyomino.x.checked_add_signed(face.dx);
            let cy = polyomino.y.checked_add_signed(face.dy);
            let x = cx.ok_or("Polyomino cannot extend past the grid bounds")? as usize;
            let y = cy.ok_or("Polyomino cannot extend past the grid bounds")? as usize;
            let index = y * width + x;

            faces[index] = Some(FaceRef {
                polyomino: rc.clone(),
                index: i as u8,
            });
        }
    }

    let hand = level.hand.clone().into_iter().map(Rc::new).collect();

    Ok(Board {
        width: width as u8,
        height: height as u8,
        cells,
        faces,
        hand,
    })
}

pub fn load(name: &str) -> Result<Board, Box<dyn Error>> {
    let file_name = format!("levels/{name}.json");
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    let level = serde_json::from_reader(reader)?;
    let board = create_board(&level)?;
    Ok(board)
}
