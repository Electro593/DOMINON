use std::{error::Error, fmt};

use crate::{level::Level, ringbuffer::RingBuffer};

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
    original: Level,
    history: RingBuffer<Level, 128>,
    cursor: usize,
}

impl Engine {
    pub fn new(level: Level) -> Self {
        Self {
            original: level,
            history: const { RingBuffer::new() },
            cursor: 0,
        }
    }

    pub fn current(&self) -> &Level {
        self.get(self.cursor).unwrap()
    }

    pub fn undo(&mut self) -> Result<(), EngineError> {
        if self.cursor == 0 {
            Err(EngineError::CannotUndo)
        } else {
            self.cursor -= 1;
            Ok(())
        }
    }

    pub fn redo(&mut self) -> Result<(), EngineError> {
        if self.cursor >= self.history.len() {
            Err(EngineError::CannotRedo)
        } else {
            self.cursor += 1;
            Ok(())
        }
    }

    pub fn reset(&mut self) -> Result<(), EngineError> {
        if self.cursor > 0 {
            self.push_mut(self.original.clone());
        }
        Ok(())
    }

    pub fn rotate(&mut self, hand_index: usize, clockwise: bool) -> Result<(), EngineError> {
        let mut level = self.current().clone();

        let polyomino = level
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

        self.push_mut(level);
        Ok(())
    }

    pub fn place(&mut self, hand_index: usize, x: usize, y: usize) -> Result<(), EngineError> {
        let mut level = self.current().clone();

        if hand_index >= level.hand.len() {
            return Err(EngineError::PolyominoNotFound);
        }

        let mut polyomino = level.hand.remove(hand_index);
        polyomino.x = x;
        polyomino.y = y;

        for face in polyomino.faces.iter() {
            let fx = x.wrapping_add_signed(face.dx);
            let fy = y.wrapping_add_signed(face.dy);

            if fx >= 256 || fy >= 256 {
                return Err(EngineError::PlacementOutOfBounds);
            }

            if level.face_at(fx, fy).is_some() {
                return Err(EngineError::PlacementCollision);
            }
        }

        level.polyominoes.push(polyomino);

        self.push_mut(level);
        Ok(())
    }

    fn get(&self, index: usize) -> Option<&Level> {
        (index == 0)
            .then_some(&self.original)
            .or_else(|| self.history[index - 1].as_ref())
    }

    fn push_mut(&mut self, level: Level) -> &mut Level {
        self.history.truncate(self.cursor);
        if self.cursor < self.history.capacity() {
            self.cursor += 1;
        }
        self.history.push_mut(level)
    }
}
