use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;

use serde::Deserialize;

#[derive(Default, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum RawFaceType {
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
}

impl RawFaceType {
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
pub enum RawCellType {
    #[serde(rename = " ")]
    #[default]
    None,
    #[serde(rename = "*")]
    Basic,
}

impl RawCellType {
    pub fn is_none(&self) -> bool {
        match self {
            RawCellType::None => true,
            _ => false,
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

#[derive(Deserialize, Copy, Clone)]
pub struct RawFace {
    pub symbol: RawFaceType,
    #[serde(default)]
    pub dx: isize,
    #[serde(default)]
    pub dy: isize,
}

#[derive(Deserialize, Clone)]
pub struct RawPolyomino {
    #[serde(default)]
    pub x: usize,
    #[serde(default)]
    pub y: usize,
    #[serde(default)]
    pub faces: Vec<RawFace>,
}

impl RawPolyomino {
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

    pub fn face_at(&self, dx: isize, dy: isize) -> RawFaceType {
        self.faces
            .iter()
            .find(|f| f.dx == dx && f.dy == dy)
            .map(|f| f.symbol)
            .unwrap_or(RawFaceType::None)
    }
}

#[derive(Deserialize, Clone)]
pub struct Level {
    pub polyominoes: Vec<RawPolyomino>,
    pub hand: Vec<RawPolyomino>,
    pub cells: Vec<Vec<RawCellType>>,
}

impl Level {
    pub fn is_won(&self) -> bool {
        self.hand.len() == 0 && self.polyominoes.len() == 0
    }

    pub fn is_lost(&self) -> bool {
        self.hand.len() == 0 && self.polyominoes.len() > 0
    }

    pub fn bounds(&self) -> (usize, usize) {
        let w = self.cells.iter().map(|r| r.len()).max().unwrap_or_default();
        let h = self.cells.len();
        (w, h)
    }

    pub fn cell_at(&self, x: usize, y: usize) -> RawCellType {
        *self
            .cells
            .get(y)
            .unwrap_or(&vec![])
            .get(x)
            .unwrap_or(&RawCellType::None)
    }

    pub fn face_at(&self, x: usize, y: usize) -> Option<(&RawFace, &RawPolyomino)> {
        for p in self.polyominoes.iter() {
            for f in p.faces.iter() {
                if p.x.wrapping_add_signed(f.dx) == x && p.y.wrapping_add_signed(f.dy) == y {
                    return Some((f, p));
                }
            }
        }
        None
    }
}

pub fn list() -> Vec<String> {
    let paths = fs::read_dir("levels");
    if paths.is_err() {
        return vec![];
    }

    let mut names: Vec<String> = vec![];
    for dir_entry in paths.unwrap() {
        if let Ok(d) = dir_entry {
            let mut path = d.path();
            if path.is_file() && path.set_extension("") {
                names.push(path.file_name().unwrap().to_string_lossy().into())
            }
        }
    }

    names
}

pub fn load(name: &String) -> Result<Level, Box<dyn Error>> {
    let file_name = format!("levels/{name}.json");
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    let level = serde_json::from_reader(reader)?;
    Ok(level)
}
