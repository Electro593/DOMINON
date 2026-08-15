mod board;
mod engine;
mod level;
mod ringbuffer;
mod tui;

use std::{fmt, io, iter, mem::discriminant};

use crossterm;

use crate::{
    engine::Engine,
    level::{Level, RawCellType, RawFaceType, RawPolyomino},
};

struct AlphabetCounter {
    count: usize,
}

impl AlphabetCounter {
    fn new() -> Self {
        AlphabetCounter { count: 1 }
    }

    fn from(count: usize) -> Option<String> {
        if count == 0 {
            return None;
        }

        let mut value: Vec<char> = vec!['\0'; 14];
        let mut c = count;
        let mut i = 14;

        while c > 0 {
            i -= 1;
            let n = c - 1;
            value[i] = (b'A' + (n % 26) as u8) as char;
            c = n / 26;
        }

        Some(String::from_iter(&value[i..14]))
    }

    fn parse(str: &String) -> Option<usize> {
        let mut count: usize = 0;

        for c in str.to_ascii_uppercase().chars() {
            if !c.is_ascii_alphabetic() {
                return None;
            }

            count = count * 26 + 1;
            count += c.to_ascii_uppercase() as usize - 'A' as usize;
        }

        (count != 0).then_some(count)
    }
}

impl Iterator for AlphabetCounter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let result = Self::from(self.count);
        self.count += 1;
        result
    }
}

fn face_type_to_string(face: RawFaceType) -> &'static str {
    match face {
        RawFaceType::None => "    ",
        RawFaceType::Zero => "  0 ",
        RawFaceType::One => "  1 ",
        RawFaceType::Two => "  2 ",
        RawFaceType::Three => "  3 ",
        RawFaceType::Four => "  4 ",
        RawFaceType::Five => "  5 ",
        RawFaceType::Six => "  6 ",
        RawFaceType::Seven => "  7 ",
        RawFaceType::Eight => "  8 ",
        RawFaceType::Nine => "  9 ",
        RawFaceType::Ten => " 10 ",
        RawFaceType::Eleven => " 11 ",
        RawFaceType::Twelve => " 12 ",
    }
}

fn cell_type_to_string(cell: RawCellType) -> &'static str {
    match cell {
        RawCellType::None => "    ",
        RawCellType::Basic => "    ",
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TextCell {
    Corner(TextCellCorner),
    HLine(TextCellHLine),
    VLine(TextCellVLine),
    Face(RawFaceType),
    Cell(RawCellType),
    Pivot,
}

impl TextCell {
    fn grid_offsets(&self) -> Vec<(usize, usize)> {
        match self {
            Self::Corner(_) => vec![(0, 0), (0, 3), (2, 0), (2, 3)],
            Self::HLine(_) => vec![(1, 0), (1, 3)],
            Self::VLine(_) => vec![(0, 1), (0, 2), (2, 1), (2, 2)],
            Self::Face(_) => vec![(1, 1)],
            Self::Cell(_) => vec![(1, 2)],
            Self::Pivot => vec![(1, 2)],
        }
    }

    fn promote(self, cell: TextCell) -> Option<TextCell> {
        if self == cell {
            return Some(self);
        }

        if self < cell {
            return cell.promote(self);
        }

        if discriminant(&self) == discriminant(&cell) {
            return Some(self);
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
                Self::Pivot => "|==|".into(),
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
            push_row(vline, TextCell::Face(RawFaceType::None));
            push_row(vline, TextCell::Cell(RawCellType::None));
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

    fn set_all(&mut self, x: usize, y: usize, cell: TextCell) {
        cell.grid_offsets()
            .iter()
            .for_each(|(dx, dy)| self.set(x * 2 + dx, y * 3 + dy, cell));
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
    let bounds = level.hand.iter().map(RawPolyomino::bounds);
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

            if f.dx == 0 && f.dy == 0 {
                grid.set_all(x, y, TextCell::Pivot);
            }

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
          Repeat the last action, or display help if none have been done yet.
    h     Print this help text.
    q     Quit the game.
    w     Display the current state of the game.
    e     Load a new level.
    aN    Rotate polyomino N by 90 degrees counter-clockwise.
    sNXY  Set polyomino N at board cell XY (X is alphabetical, Y is numerical).
    dN    Rotate polyomino N by 90 degrees clockwise.
    z     Undo the last action.
    x     Clear the level to its initial state. This can be undone.
    c     Redo the last undo.
";

#[derive(Clone)]
enum GameAction {
    RepeatLast,
    Help,
    Quit,
    SwitchLevel,
    NextLevel,
    ShowBoard,
    Reset,
    Redo,
    Undo,
    Rotate { i: usize, cw: bool },
    Place { i: usize, x: usize, y: usize },
}

impl GameAction {
    fn parse(input: String) -> Result<GameAction, String> {
        let lower = input.to_ascii_lowercase();
        let ref mut chars = lower.chars().peekable();

        let command = chars.next();
        let action = match command {
            None => Ok(Self::RepeatLast),
            Some('h') => (chars.peek().is_none() || chars.collect::<String>() == "elp")
                .then_some(Self::Help)
                .ok_or(None),
            Some('q') => Ok(Self::Quit),
            Some('w') => Ok(Self::ShowBoard),
            Some('e') => Ok(Self::SwitchLevel),
            Some('z') => Ok(Self::Undo),
            Some('x') => Ok(Self::Reset),
            Some('c') => Ok(Self::Redo),
            Some('a') | Some('d') => {
                let cw = command.unwrap() == 'd';

                let ir = chars
                    .collect::<String>()
                    .parse::<usize>()
                    .map(|i| i - 1)
                    .map_err(|e| Some(format!("Invalid format for N: {e}.")));

                ir.map(|i| Self::Rotate { i, cw })
            }
            Some('s') => {
                let nc = chars.clone().take_while(|c| c.is_ascii_digit());
                let xc = chars
                    .clone()
                    .skip(nc.clone().count())
                    .take_while(|c| c.is_ascii_alphabetic());
                let yc = chars.skip(nc.clone().count() + xc.clone().count());

                let ir = nc
                    .collect::<String>()
                    .parse::<usize>()
                    .map_err(|_| "Invalid format for N.")
                    .and_then(|n| n.checked_sub(1).ok_or("N must be at least 1"));

                let xr = AlphabetCounter::parse(&xc.collect::<String>())
                    .map(|n| n - 1)
                    .ok_or("Invalid format for X.");

                let yr = yc
                    .collect::<String>()
                    .parse::<usize>()
                    .map_err(|_| "Invalid format for Y.")
                    .and_then(|n| n.checked_sub(1).ok_or("Y must be at least 1"));

                ir.and_then(|i| xr.and_then(|x| yr.map(|y| Self::Place { i, x, y })))
                    .map_err(|e| Some(String::from(e)))
            }
            _ => Err(None),
        };

        action
            .and_then(|a| chars.next().is_none().then_some(a).ok_or(None))
            .map_err(|e| e.unwrap_or("Command not recognized.".into()))
    }
}

fn read_line() -> Option<String> {
    let mut input = String::new();
    input.clear();
    if let Err(e) = io::stdin().read_line(&mut input) {
        println!("Failed to read from stdio: {e}");
        None
    } else {
        Some(input)
    }
}

fn clear_terminal() {
    let _ = crossterm::execute!(
        io::stdout(),
        crossterm::cursor::MoveTo(0, 0),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::Purge)
    );
}

fn play(engine: &mut Engine, level_name: &String) -> GameAction {
    let mut last_action = None;

    loop {
        let level = engine.current();

        if level.is_won() {
            return GameAction::NextLevel;
        }

        clear_terminal();
        println!("LEVEL {level_name}\n\nBOARD\n");
        display_board(level);
        println!("\nHAND\n");
        display_hand(level);
        println!();

        if level.is_lost() {
            println!("No more moves! Undo or reset to continue.");
        }

        loop {
            println!("What would you like to do?");

            let input = read_line();
            if let None = input {
                return GameAction::Quit;
            }

            let action = GameAction::parse(input.unwrap().trim().into());

            if let Err(e) = action {
                println!("{e}");
                continue;
            }

            let resolved_action = match action.unwrap() {
                GameAction::RepeatLast => last_action.clone().unwrap_or(GameAction::Help),
                a => {
                    last_action = Some(a.clone());
                    a
                }
            };

            let get_confirmation = || {
                read_line()
                    .map(|s| s.trim().eq_ignore_ascii_case("y"))
                    .unwrap_or(true)
            };

            match resolved_action {
                GameAction::RepeatLast => continue,
                GameAction::Help => println!("{HELP_STRING}"),
                GameAction::ShowBoard => break,
                GameAction::Reset => {
                    if let Err(e) = engine.reset() {
                        println!("{e}.");
                    } else {
                        break;
                    }
                }
                GameAction::Redo => {
                    if let Err(e) = engine.redo() {
                        println!("{e}.");
                    } else {
                        break;
                    }
                }
                GameAction::Undo => {
                    if let Err(e) = engine.undo() {
                        println!("{e}.");
                    } else {
                        break;
                    }
                }
                GameAction::Rotate { i, cw } => {
                    if let Err(e) = engine.rotate(i, cw) {
                        println!("{e}.");
                    } else {
                        break;
                    }
                }
                GameAction::Place { i, x, y } => {
                    if let Err(e) = engine.place(i, x, y) {
                        println!("{e}.");
                    } else {
                        break;
                    }
                }
                GameAction::SwitchLevel | GameAction::NextLevel => {
                    println!("\nAre you sure you want to leave this puzzle? (y/N)");
                    if get_confirmation() {
                        return resolved_action;
                    }
                }
                GameAction::Quit => {
                    println!("\nAre you sure you want to exit the game? (y/N)");
                    if get_confirmation() {
                        return resolved_action;
                    }
                }
            };
        }
    }
}

fn load_level(levels: &Vec<String>, index: Option<usize>) -> Option<(Level, usize)> {
    if levels.is_empty() {
        println!("No levels found! Are you running this in the correct directory?");
        return None;
    }

    let mut first = true;
    let mut index_mut = index;

    loop {
        match index_mut {
            Some(i) => {
                if i >= levels.len() {
                    println!("Congrats, you win!\n");
                    return None;
                }

                match level::load(&levels[i]) {
                    Ok(level) => return Some((level, i)),
                    Err(e) => {
                        println!("Failed to load level: {e}");
                        index_mut = None;
                    }
                }
            }
            None => {
                println!("Which level would you like to play?");

                if first {
                    println!("Available: {}", levels.join(" "));
                    first = false;
                }

                let input = read_line().unwrap_or(String::from("q"));
                let trimmed = input.trim();
                if trimmed == "q" {
                    return None;
                }

                match levels.iter().position(|l| l.eq(&String::from(trimmed))) {
                    Some(i) => index_mut = Some(i),
                    None => println!("Level must be one of the available options."),
                }
            }
        }
    }
}

fn main() {
    crate::tui::start();
    return;

    println!("Welcome to DOMINON!");

    let levels = level::list();
    // let mut level_index = Some(0);
    let mut level_index = None;

    loop {
        match load_level(&levels, level_index) {
            Some((level, index)) => {
                let mut engine = Engine::new(level);

                match play(&mut engine, &levels[index]) {
                    GameAction::Quit => return,
                    GameAction::SwitchLevel => level_index = None,
                    GameAction::NextLevel => level_index = Some(index + 1),
                    _ => (),
                }

                clear_terminal();
            }
            None => return,
        }
    }
}
