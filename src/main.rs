use std::error::Error;
use std::fmt;

mod level;

use crate::level::{CellType, FaceType, Level, Polyomino};

fn face_type_to_string(face: FaceType) -> &'static str {
    match face {
        FaceType::None => "    ",
        FaceType::Zero => "  0 ",
        FaceType::One => "  1 ",
        FaceType::Two => "  2 ",
        FaceType::Three => "  3 ",
        FaceType::Four => "  4 ",
        FaceType::Five => "  5 ",
        FaceType::Six => "  6 ",
        FaceType::Seven => "  7 ",
        FaceType::Eight => "  8 ",
        FaceType::Nine => "  9 ",
        FaceType::Ten => " 10 ",
        FaceType::Eleven => " 11 ",
        FaceType::Twelve => " 12 ",
        FaceType::Mirror => " ?? ",
        FaceType::Boom => " ** ",
    }
}

fn cell_type_to_string(cell: CellType) -> &'static str {
    match cell {
        CellType::None => "    ",
        CellType::Basic => "    ",
        CellType::SlideUp => " ^^ ",
        CellType::SlideDown => " VV ",
        CellType::SlideLeft => " <- ",
        CellType::SlideRight => " -> ",
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TextCellCorner {
    None,
    Present,
}

impl fmt::Display for TextCellCorner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "   ",
                Self::Present => " + ",
            }
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TextCellHLine {
    None,
    Partial,
    Solid,
    Connected,
}

impl fmt::Display for TextCellHLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "    ",
                Self::Partial => "-  -",
                Self::Solid => "----",
                Self::Connected => "    ",
            }
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TextCellVLine {
    None,
    Partial,
    Solid,
    Connected,
}

impl fmt::Display for TextCellVLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "   ",
                Self::Partial => " : ",
                Self::Solid => " | ",
                Self::Connected => "   ",
            }
        )
    }
}

#[derive(Copy, Clone)]
enum TextCell {
    Corner(TextCellCorner),
    HLine(TextCellHLine),
    VLine(TextCellVLine),
    Face(FaceType),
    Cell(CellType),
}

impl TextCell {
    fn grid_offsets(&self) -> Vec<(usize, usize)> {
        match self {
            Self::Corner(_) => vec![(0, 0), (0, 3), (2, 0), (2, 3)],
            Self::HLine(_) => vec![(1, 0), (1, 3)],
            Self::VLine(_) => vec![(0, 1), (0, 2), (2, 1), (2, 2)],
            Self::Face(_) => vec![(1, 1)],
            Self::Cell(_) => vec![(1, 2)],
        }
    }

    fn promote(self, cell: TextCell) -> Option<TextCell> {
        if let Self::Corner(c1) = self
            && let Self::Corner(c2) = cell
        {
            return Some(Self::Corner(c1.max(c2)));
        }
        if let Self::HLine(c1) = self
            && let Self::HLine(c2) = cell
        {
            return Some(Self::HLine(c1.max(c2)));
        }
        if let Self::VLine(c1) = self
            && let Self::VLine(c2) = cell
        {
            return Some(Self::VLine(c1.max(c2)));
        }
        if let Self::Face(c1) = self
            && let Self::Face(c2) = cell
        {
            return Some(Self::Face(c1.max(c2)));
        }
        if let Self::Cell(c1) = self
            && let Self::Cell(c2) = cell
        {
            return Some(Self::Cell(c1.max(c2)));
        }
        None
    }
}

impl fmt::Display for TextCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Corner(corner) => corner.to_string(),
                Self::HLine(hline) => hline.to_string(),
                Self::VLine(vline) => vline.to_string(),
                Self::Face(face) => face_type_to_string(*face).to_string(),
                Self::Cell(cell) => cell_type_to_string(*cell).to_string(),
            }
        )
    }
}

struct TextGrid {
    grid: Vec<Vec<TextCell>>,
}

impl TextGrid {
    fn new(w: usize, h: usize) -> Self {
        let grid_width = w * 2 + 1;
        let grid_height = h * 3 + 1;
        let mut grid = Vec::with_capacity(grid_height);

        let mut push_row = |b, c| {
            let r = grid.push_mut(Vec::with_capacity(grid_width));
            for _ in 0..w {
                r.push(b);
                r.push(c);
            }
            r.push(b);
        };

        let corner = TextCell::Corner(TextCellCorner::None);
        let hline = TextCell::HLine(TextCellHLine::None);
        let vline = TextCell::VLine(TextCellVLine::None);

        for _ in 0..h {
            push_row(TextCell::Corner(TextCellCorner::None), hline);
            push_row(vline, TextCell::Face(FaceType::None));
            push_row(vline, TextCell::Cell(CellType::None));
        }
        push_row(corner, hline);

        TextGrid { grid }
    }

    fn get(&self, gx: usize, gy: usize) -> Option<TextCell> {
        self.grid.get(gy).and_then(|r| r.get(gx)).copied()
    }

    fn set(&mut self, gx: usize, gy: usize, cell: TextCell) {
        self.grid[gy][gx] = cell;
    }

    fn promote(&mut self, gx: usize, gy: usize, cell: TextCell) {
        self.get(gx, gy)
            .and_then(|c| c.promote(cell))
            .map(|c| self.set(gx, gy, c));
    }

    fn promote_all(&mut self, x: usize, y: usize, cell: TextCell) {
        cell.grid_offsets()
            .iter()
            .for_each(|(dx, dy)| self.promote(x * 2 + dx, y * 3 + dy, cell));
    }
}

impl fmt::Display for TextGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.grid
                .iter()
                .map(|r| r
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(""))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

fn display_board(level: &Level) {
    let (w, h) = level.bounds();
    let mut grid = TextGrid::new(w, h);

    for y in 0..h {
        for x in 0..w {
            grid.promote_all(x, y, TextCell::Corner(TextCellCorner::Present));

            let cell = level.cell_at(x, y);
            if cell.is_some() {
                grid.promote_all(x, y, TextCell::HLine(TextCellHLine::Partial));
                grid.promote_all(x, y, TextCell::VLine(TextCellVLine::Partial));
                grid.promote_all(x, y, TextCell::Cell(cell));
            }

            if let Some((f, p)) = level.face_at(x, y) {
                grid.promote_all(x, y, TextCell::HLine(TextCellHLine::Solid));
                grid.promote_all(x, y, TextCell::VLine(TextCellVLine::Solid));
                grid.promote_all(x, y, TextCell::Face(f.symbol));

                if p.face_at(f.dx - 1, f.dy).is_some() {
                    grid.promote(x * 2, y * 3 + 1, TextCell::VLine(TextCellVLine::Connected));
                    grid.promote(x * 2, y * 3 + 2, TextCell::VLine(TextCellVLine::Connected));
                }
                if p.face_at(f.dx, f.dy - 1).is_some() {
                    grid.promote(x * 2 + 1, y * 3, TextCell::HLine(TextCellHLine::Connected));
                }
            }
        }
    }

    println!("{grid}");
}

fn display_hand(level: &Level) {
    let bounds = level.hand.iter().map(Polyomino::bounds);
    let x0s = bounds.clone().map(|b| b.0);
    let y0s = bounds.clone().map(|b| b.1);
    let x1s = bounds.clone().map(|b| b.2);
    let y1s = bounds.clone().map(|b| b.3);

    let y_min = y0s.min().unwrap_or_default();
    let y_max = y1s.max().unwrap_or_default();

    let ws = x0s.clone().zip(x1s).map(|(x0, x1)| x1.abs_diff(x0) + 1);
    let w = ws.clone().sum::<usize>() + level.hand.len() - 1;
    let h = y_max.abs_diff(y_min) + 1;

    let mut grid = TextGrid::new(w, h);

    let mut px: usize = 0;
    for (p, (x0, pw)) in level.hand.iter().zip(x0s.zip(ws)) {
        for f in p.faces.iter() {
            let x = px + f.dx.abs_diff(x0);
            let y = f.dy.abs_diff(y_min);

            grid.promote_all(x, y, TextCell::Corner(TextCellCorner::Present));
            grid.promote_all(x, y, TextCell::Face(f.symbol));
            grid.promote_all(x, y, TextCell::HLine(TextCellHLine::Solid));
            grid.promote_all(x, y, TextCell::VLine(TextCellVLine::Solid));

            if p.face_at(f.dx - 1, f.dy).is_some() {
                grid.promote(x * 2, y * 3 + 1, TextCell::VLine(TextCellVLine::Connected));
                grid.promote(x * 2, y * 3 + 2, TextCell::VLine(TextCellVLine::Connected));
            }
            if p.face_at(f.dx, f.dy - 1).is_some() {
                grid.promote(x * 2 + 1, y * 3, TextCell::HLine(TextCellHLine::Connected));
            }
        }

        px += pw + 1;
    }

    println!("{grid}");
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome to DOMINON!");

    let board = level::load("1")?;

    println!("\nBOARD");
    display_board(&board);

    println!("\nHAND");
    display_hand(&board);

    Ok(())
}
