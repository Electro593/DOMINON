use std::fmt;
use std::io;
use std::iter;
use std::process::ExitCode;

mod engine;
mod level;

use crate::engine::Engine;
use crate::level::{CellType, FaceType, Level, Polyomino};

struct AlphabetCounter {
    count: usize,
}

impl AlphabetCounter {
    fn new() -> Self {
        AlphabetCounter { count: 1 }
    }

    fn parse(str: &String) -> Result<usize, &'static str> {
        let mut count: usize = 0;

        for c in str.to_ascii_uppercase().chars() {
            if !c.is_ascii_alphabetic() {
                return Err("Expected a series of letters");
            }

            count = count * 26 + 1;
            count += c.to_ascii_uppercase() as usize - 'A' as usize;
        }

        Ok(count)
    }
}

impl Iterator for AlphabetCounter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        }

        let mut value: Vec<char> = vec!['\0'; 14];
        let mut i = 14;

        let mut count = self.count;
        self.count += 1;

        while count > 0 {
            i -= 1;
            let n = count - 1;
            value[i] = (b'A' + (n % 26) as u8) as char;
            count = n / 26;
        }

        Some(String::from_iter(&value[i..14]))
    }
}

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
    x_labels: Vec<String>,
    y_labels: Vec<String>,
    grid: Vec<Vec<TextCell>>,
}

impl TextGrid {
    fn new(x_labels: Vec<String>, y_labels: Vec<String>) -> Self {
        let w = x_labels.len();
        let h = y_labels.len();

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

        TextGrid {
            x_labels,
            y_labels,
            grid,
        }
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
        let mut x_label_row = self
            .x_labels
            .iter()
            .map(|l| format!("{:^7}", l.chars().take(6).collect::<String>()))
            .collect::<String>();
        x_label_row = format!("  {x_label_row}");

        let y_labels = self
            .y_labels
            .iter()
            .flat_map(|l| vec!["", "", l])
            .chain(vec!["", ""]);

        let content_rows = self
            .grid
            .iter()
            .map(|r| {
                r.iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            })
            .chain(iter::once(x_label_row));

        let labelled_rows = content_rows.zip(y_labels);

        let y_label_width = self.y_labels.iter().map(String::len).max().unwrap_or(0);

        let text = labelled_rows
            .map(|(r, l)| format!("{l:<y_label_width$}{r}"))
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{text}")
    }
}

fn display_board(level: &Level) {
    let (w, h) = level.bounds();
    let mut grid = TextGrid::new(
        AlphabetCounter::new().take(w).collect(),
        (1..=h).map(|n| n.to_string()).collect(),
    );

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

    let xs = x0s.clone().zip(x1s);
    let ws = xs.clone().map(|(x0, x1)| x1.abs_diff(x0) + 1);
    let h = y_max.abs_diff(y_min) + 1;

    let mut grid = TextGrid::new(
        xs.enumerate()
            .flat_map(|(i, (x0, x1))| (x0..x1 + 2).map(move |n| (i, n)))
            .map(|(i, n)| {
                if n == 0 {
                    (i + 1).to_string()
                } else {
                    String::new()
                }
            })
            .collect(),
        vec![String::new(); h],
    );

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

const HELP_STRING: &str = "Available commands:
    q     Exit the game.
    h     Print this help text.
    lL    Load a new level L. This will delete your current progress!
    x     Reset the level to its initial state. This can be undone.
    y     Redo the last undo.
    z     Undo the last action.
    cwN   Rotate polyomino N by 90 degrees clockwise.
    ccwN  Rotate polyomino N by 90 degrees counter-clockwise.
    pNXY  Move polyomino N to board cell XY (X is alphabetical, Y is numerical).";

fn play(engine: &mut Engine) -> Option<String> {
    loop {
        println!("\nBOARD");
        display_board(&engine.level);

        println!("\nHAND");
        display_hand(&engine.level);

        loop {
            println!("\nWhat would you like to do?");
            let mut input = String::new();
            input.clear();
            if let Err(e) = io::stdin().read_line(&mut input) {
                println!("Failed to read from stdio. {e}");
                continue;
            }

            let action = input.trim().to_ascii_lowercase();
            let mut chars = action.chars().peekable();
            match chars.next() {
                None => {
                    println!("{HELP_STRING}");
                    continue;
                }
                Some('q') => {
                    if chars.next().is_some() {
                        println!("Command not recognized.");
                        continue;
                    }

                    return None;
                }
                Some('h') => {
                    if chars.next().is_some() {
                        println!("Command not recognized.");
                        continue;
                    }

                    println!("{HELP_STRING}")
                }
                Some('l') => return Some(chars.collect::<String>()),
                Some('x') => {
                    if chars.next().is_some() {
                        println!("Command not recognized.");
                        continue;
                    }

                    if let Err(e) = engine.reset() {
                        println!("Cannot reset: {e}.");
                        continue;
                    }

                    println!("Reset the board.");
                    break;
                }
                Some('y') => {
                    if chars.next().is_some() {
                        println!("Command not recognized.");
                        continue;
                    }

                    if let Err(e) = engine.redo() {
                        println!("Cannot redo: {e}.");
                        continue;
                    }

                    println!("Redid one action.");
                    break;
                }
                Some('z') => {
                    if chars.next().is_some() {
                        println!("Command not recognized.");
                        continue;
                    }

                    if let Err(e) = engine.undo() {
                        println!("Cannot undo: {e}.");
                        continue;
                    }

                    println!("Undid one action.");
                    break;
                }
                Some('c') => {
                    let cw = chars.next_if(|c| *c == 'c').is_none();
                    if chars.next_if(|c| *c == 'w').is_none() {
                        println!("Command not recognized.");
                        continue;
                    }

                    let ns = chars.collect::<String>();
                    let n = ns.parse::<usize>();

                    if let Err(e) = n {
                        println!("Invalid format for N: {e}.");
                        continue;
                    }

                    if let Err(e) = engine.rotate(n.unwrap() - 1, cw) {
                        println!("Cannot rotate piece: {e}.");
                        continue;
                    }

                    println!("Rotated piece {ns}.");
                    break;
                }
                Some('p') => {
                    let nc = chars.clone().take_while(|c| c.is_ascii_digit());
                    let xc = chars
                        .clone()
                        .skip(nc.clone().count())
                        .take_while(|c| c.is_ascii_alphabetic());
                    let yc = chars.skip(nc.clone().count() + xc.clone().count());

                    let ns = nc.collect::<String>();
                    let xs = xc.collect::<String>();
                    let ys = yc.collect::<String>();

                    let n = ns.parse::<usize>();
                    let x = AlphabetCounter::parse(&xs);
                    let y = ys.parse::<usize>();

                    if let Err(e) = n {
                        println!("Invalid format for N: {e}.");
                        continue;
                    }
                    if let Err(e) = x {
                        println!("Invalid format for X: {e}.");
                        continue;
                    }
                    if let Err(e) = y {
                        println!("Invalid format for Y: {e}.");
                        continue;
                    }

                    if let Err(e) = engine.place(n.unwrap() - 1, x.unwrap() - 1, y.unwrap() - 1) {
                        println!("Cannot place piece: {e}.");
                        continue;
                    }

                    println!("Placed piece {ns} at {xs}{ys}.");
                    break;
                }
                _ => println!("Command not recognized."),
            };
        }
    }
}

fn main() -> ExitCode {
    println!("Welcome to DOMINON!");

    let mut level_name = Some(String::from("1"));

    loop {
        if let None = level_name {
            println!("\nWhich level would you like to play?");

            let mut input = String::new();
            input.clear();
            if let Err(e) = io::stdin().read_line(&mut input) {
                println!("Failed to read from stdio. {e}");
            } else {
                level_name = Some(String::from(input.trim()));
            }
        } else if let Some(ref ln) = level_name {
            let level = level::load(ln);
            if let Err(e) = level {
                println!("Failed to load level: {e}");
                level_name = None;
                continue;
            }

            println!("\nLEVEL {ln}");
            let mut engine = Engine::new(level.unwrap());

            if let Some(n) = play(&mut engine) {
                level_name = Some(n);
                continue;
            } else {
                return ExitCode::SUCCESS;
            }
        }
    }
}
