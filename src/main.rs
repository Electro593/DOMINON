use std::error::Error;

mod level;
use level::{Board, CellType, FaceType};

fn display_board(board: &Board) {
    let mut result = String::new();

    let hborder = |x: u8, y0: u8, y1: u8| {
        if board.face_at(x, y0).is_some() || board.face_at(x, y1).is_some() {
            "------"
        } else if board.cell_at(x, y0).is_some() || board.cell_at(x, y1).is_some() {
            " -  - "
        } else {
            "      "
        }
    };

    let vborder = |x0: u8, x1: u8, y: u8| {
        if board.face_at(x0, y).is_some() || board.face_at(x1, y).is_some() {
            "|"
        } else if board.cell_at(x0, y).is_some() || board.cell_at(x1, y).is_some() {
            "."
        } else {
            " "
        }
    };

    let top_line = |x: u8, y: u8| match board.face_at(x, y) {
        None => "      ",
        Some(face) => match face {
            FaceType::Zero => "   0  ",
            FaceType::One => "   1  ",
            FaceType::Two => "   2  ",
            FaceType::Three => "   3  ",
            FaceType::Four => "   4  ",
            FaceType::Five => "   5  ",
            FaceType::Six => "   6  ",
            FaceType::Seven => "   7  ",
            FaceType::Eight => "   8  ",
            FaceType::Nine => "   9  ",
            FaceType::Ten => "  10  ",
            FaceType::Eleven => "  11  ",
            FaceType::Twelve => "  12  ",
            FaceType::Mirror => "  ??  ",
            FaceType::Boom => "  **  ",
        },
    };

    let bottom_line = |x: u8, y: u8| match board.cell_at(x, y) {
        None => "      ",
        Some(cell) => match cell {
            CellType::None => "      ",
            CellType::Basic => "      ",
            CellType::SlideUp => "  ^^  ",
            CellType::SlideDown => "  VV  ",
            CellType::SlideLeft => "  <-  ",
            CellType::SlideRight => "  ->  ",
        },
    };

    for y in 0..=board.height {
        for x in 0..=board.width {
            result.push_str("+");
            if x < board.width {
                result.push_str(hborder(x, y.wrapping_sub(1), y));
            }
        }
        result.push_str("\n");

        if y < board.height {
            for x in 0..=board.width {
                result.push_str(vborder(x.wrapping_sub(1), x, y));
                if x < board.width {
                    result.push_str(top_line(x, y));
                }
            }
            result.push_str("\n");

            for x in 0..=board.width {
                result.push_str(vborder(x.wrapping_sub(1), x, y));
                if x < board.width {
                    result.push_str(bottom_line(x, y));
                }
            }
            result.push_str("\n");
        }
    }

    println!("{result}");
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome to DOMINON!");

    let board = level::load("1")?;
    display_board(&board);

    Ok(())
}
