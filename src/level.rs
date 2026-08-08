use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

#[derive(Default, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum FaceType {
    #[serde(skip)]
    #[default]
    None,
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
    One,
    #[serde(rename = "2")]
    Two,
    #[serde(rename = "3")]
    Three,
    #[serde(rename = "4")]
    Four,
    #[serde(rename = "5")]
    Five,
    #[serde(rename = "6")]
    Six,
    #[serde(rename = "7")]
    Seven,
    #[serde(rename = "8")]
    Eight,
    #[serde(rename = "9")]
    Nine,
    #[serde(rename = "10")]
    Ten,
    #[serde(rename = "11")]
    Eleven,
    #[serde(rename = "12")]
    Twelve,
    #[serde(rename = "?")]
    Mirror,
    #[serde(rename = "*")]
    Boom,
}

impl FaceType {
    pub fn is_none(&self) -> bool {
        match self {
            FaceType::None => true,
            _ => false,
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

#[derive(Default, Deserialize, Copy, Clone)]
pub enum CellType {
    #[serde(rename = " ")]
    #[default]
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

impl CellType {
    pub fn is_none(&self) -> bool {
        match self {
            CellType::None => true,
            _ => false,
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

#[derive(Deserialize, Copy, Clone)]
pub struct Face {
    pub symbol: FaceType,
    #[serde(default)]
    pub dx: isize,
    #[serde(default)]
    pub dy: isize,
}

#[derive(Deserialize, Clone)]
pub struct Polyomino {
    #[serde(default)]
    pub x: usize,
    #[serde(default)]
    pub y: usize,
    #[serde(default)]
    pub shield: usize,
    pub faces: Vec<Face>,
}

impl Polyomino {}

#[derive(Deserialize, Clone)]
pub struct Level {
    pub polyominoes: Vec<Polyomino>,
    pub hand: Vec<Polyomino>,
    pub cells: Vec<Vec<CellType>>,
}

impl Level {
    pub fn bounds(&self) -> (usize, usize) {
        (
            self.cells.iter().map(|r| r.len()).max().unwrap_or_default(),
            self.cells.len(),
        )
    }

    pub fn cell_at(&self, x: usize, y: usize) -> CellType {
        *self
            .cells
            .get(y)
            .unwrap_or(&vec![])
            .get(x)
            .unwrap_or(&CellType::None)
    }

    pub fn face_at(&self, x: usize, y: usize) -> FaceType {
        self.polyominoes
            .iter()
            .flat_map(|p| p.faces.iter().map(|f| (f, p.x, p.y)))
            .find(|(f, px, py)| {
                px.wrapping_add_signed(f.dx) == x && py.wrapping_add_signed(f.dy) == y
            })
            .map(|(f, _, _)| f.symbol)
            .unwrap_or_default()
    }
}

pub fn load(name: &str) -> Result<Level, Box<dyn Error>> {
    let file_name = format!("levels/{name}.json");
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    let level = serde_json::from_reader(reader)?;
    Ok(level)
}
