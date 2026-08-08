use std::error::Error;

mod level;
mod util;

use level::{CellType, FaceType, Level};

fn display_board(level: &Level) {
    let mut result = String::new();

    let hborder = |x, y0, y1| {
        if level.face_at(x, y0).is_some() || level.face_at(x, y1).is_some() {
            "------"
        } else if level.cell_at(x, y0).is_some() || level.cell_at(x, y1).is_some() {
            " -  - "
        } else {
            "      "
        }
    };

    let vborder = |x0, x1, y| {
        if level.face_at(x0, y).is_some() || level.face_at(x1, y).is_some() {
            "|"
        } else if level.cell_at(x0, y).is_some() || level.cell_at(x1, y).is_some() {
            "."
        } else {
            " "
        }
    };

    let top_line = |x, y| match level.face_at(x, y) {
        FaceType::None => "      ",
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
    };

    let bottom_line = |x, y| match level.cell_at(x, y) {
        CellType::None => "      ",
        CellType::Basic => "      ",
        CellType::SlideUp => "  ^^  ",
        CellType::SlideDown => "  VV  ",
        CellType::SlideLeft => "  <-  ",
        CellType::SlideRight => "  ->  ",
    };

    let (width, height) = level.bounds();

    for y in 0..=height {
        for x in 0..=width {
            result.push_str("+");
            if x < width {
                result.push_str(hborder(x, y.wrapping_sub(1), y));
            }
        }
        result.push_str("\n");

        if y < height {
            for x in 0..=width {
                result.push_str(vborder(x.wrapping_sub(1), x, y));
                if x < width {
                    result.push_str(top_line(x, y));
                }
            }
            result.push_str("\n");

            for x in 0..=width {
                result.push_str(vborder(x.wrapping_sub(1), x, y));
                if x < width {
                    result.push_str(bottom_line(x, y));
                }
            }
            result.push_str("\n");
        }
    }

    println!("{result}");
}

fn display_hand(level: &Level) {
    //     let (xs, ys) = board
    //         .hand
    //         .iter()
    //         .flat_map(|h| h.faces.iter().map(|f| (f.dx, f.dy)))
    //         .unzip();
    //     let (width, height): (u8, u8) = (xs.max(), ys.max());
    //
    //     for polyomino in board.hand.iter() {}
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
