use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

#[derive(Default, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
            Self::None => true,
            _ => false,
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

#[derive(Default, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    pub faces: Vec<Face>,
}

impl Polyomino {
    pub fn bounds(&self) -> (isize, isize, isize, isize) {
        let xs = self.faces.iter().map(|r| r.dx);
        let ys = self.faces.iter().map(|r| r.dy);
        let x0 = xs.clone().min().unwrap_or_default();
        let y0 = ys.clone().min().unwrap_or_default();
        let x1 = xs.max().unwrap_or_default();
        let y1 = ys.max().unwrap_or_default();

        let ns = vec![x0, y0, x1, y1, -x0, -x1, -y0, -y1];
        let min = *ns.iter().min().unwrap();
        let max = *ns.iter().max().unwrap();
        (min, min, max, max)
    }

    pub fn face_at(&self, dx: isize, dy: isize) -> FaceType {
        self.faces
            .iter()
            .find(|f| f.dx == dx && f.dy == dy)
            .map(|f| f.symbol)
            .unwrap_or(FaceType::None)
    }
}

#[derive(Deserialize, Clone)]
pub struct Level {
    pub polyominoes: Vec<Polyomino>,
    pub hand: Vec<Polyomino>,
    pub cells: Vec<Vec<CellType>>,
}

impl Level {
    pub fn bounds(&self) -> (usize, usize) {
        let w = self.cells.iter().map(|r| r.len()).max().unwrap_or_default();
        let h = self.cells.len();
        (w, h)
    }

    pub fn cell_at(&self, x: usize, y: usize) -> CellType {
        *self
            .cells
            .get(y)
            .unwrap_or(&vec![])
            .get(x)
            .unwrap_or(&CellType::None)
    }

    pub fn face_at(&self, x: usize, y: usize) -> Option<(Face, Polyomino)> {
        for p in self.polyominoes.iter() {
            for f in p.faces.iter() {
                if p.x.wrapping_add_signed(f.dx) == x && p.y.wrapping_add_signed(f.dy) == y {
                    return Some((*f, p.clone()));
                }
            }
        }
        None
    }
}

pub fn load(name: &String) -> Result<Level, Box<dyn Error>> {
    let file_name = format!("levels/{name}.json");
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    let level = serde_json::from_reader(reader)?;
    Ok(level)
}
