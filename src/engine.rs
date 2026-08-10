use std::{error::Error, fmt};

use crate::level::{Level, Polyomino};

#[derive(Debug)]
pub enum EngineError {
    CannotUndo,
    CannotRedo,
    PolyominoNotFound,
    PlacementOutOfBounds,
    PlacementCollision,
}

impl Error for EngineError {}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotUndo => write!(f, "No actions left to undo"),
            Self::CannotRedo => write!(f, "No actions available to redo"),
            Self::PolyominoNotFound => write!(f, "Piece does not exist"),
            Self::PlacementOutOfBounds => write!(f, "Cannot place piece outside of the board"),
            Self::PlacementCollision => write!(f, "Cannot place piece on top of another"),
        }
    }
}

pub struct Engine {
    pub original_level: Level,
    pub level: Level,
}

impl Engine {
    pub fn new(level: Level) -> Self {
        Engine {
            original_level: level.clone(),
            level,
        }
    }

    pub fn undo(&mut self) -> Result<(), EngineError> {
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), EngineError> {
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), EngineError> {
        self.level = self.original_level.clone();
        Ok(())
    }

    pub fn rotate(&mut self, hand_index: usize, clockwise: bool) -> Result<(), EngineError> {
        let polyomino = self
            .level
            .hand
            .get_mut(hand_index)
            .ok_or(EngineError::PolyominoNotFound)?;

        for face in polyomino.faces.iter_mut() {
            (face.dx, face.dy) = if clockwise {
                (-face.dy, face.dx)
            } else {
                (face.dy, -face.dx)
            };
        }
        Ok(())
    }

    pub fn place(&mut self, hand_index: usize, x: usize, y: usize) -> Result<(), EngineError> {
        if hand_index >= self.level.hand.len() {
            return Err(EngineError::PolyominoNotFound);
        }

        let mut p = self.level.hand.remove(hand_index);
        p.x = x;
        p.y = y;

        for f in p.faces.iter() {
            let x = p.x.wrapping_add_signed(f.dx);
            let y = p.y.wrapping_add_signed(f.dy);

            if x >= 256 || y >= 256 {
                return Err(EngineError::PlacementOutOfBounds);
            }

            if self.level.face_at(x, y).is_some() {
                return Err(EngineError::PlacementCollision);
            }
        }

        self.level.polyominoes.push(p);
        Ok(())
    }

    fn get_polyomino(&mut self, polyomino_index: usize) -> Result<&mut Polyomino, EngineError> {
        self.level
            .polyominoes
            .get_mut(polyomino_index)
            .ok_or(EngineError::PolyominoNotFound)
    }

    fn damage_polyomino(&mut self, polyomino_index: usize) -> Result<(), EngineError> {
        let polyomino = self.get_polyomino(polyomino_index)?;

        if polyomino.shield > 1 {
            polyomino.shield -= 1;
        } else {
            self.level.polyominoes.remove(polyomino_index);
        }

        Ok(())
    }
}
