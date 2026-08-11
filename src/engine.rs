use std::{error::Error, fmt, ptr};

use crate::{level::Level, ringbuffer::RingBuffer};

#[derive(Debug)]
pub enum EngineError {
    CannotUndo,
    CannotRedo,
    PolyominoNotFound,
    PlacementOutOfBounds,
    PlacementOverHole,
    PlacementCollision,
}

impl Error for EngineError {}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotUndo => write!(f, "No actions left to undo"),
            Self::CannotRedo => write!(f, "No actions available to redo"),
            Self::PolyominoNotFound => write!(f, "Piece does not exist"),
            Self::PlacementOutOfBounds => write!(f, "Pieces cannot go out of bounds"),
            Self::PlacementOverHole => write!(f, "Pieces cannot overlap board holes"),
            Self::PlacementCollision => write!(f, "Pieces cannot overlap each other"),
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

        let (w, h) = level.bounds();

        let mut polyomino = level.hand.remove(hand_index);
        polyomino.x = x;
        polyomino.y = y;

        for face in polyomino.faces.iter() {
            let fx = x.checked_add_signed(face.dx).unwrap_or(w);
            let fy = y.checked_add_signed(face.dy).unwrap_or(h);

            if fx >= w || fy >= h {
                return Err(EngineError::PlacementOutOfBounds);
            }
        }

        let faces = polyomino
            .faces
            .iter()
            .map(|f| (f, x.wrapping_add_signed(f.dx), y.wrapping_add_signed(f.dy)));

        for (_, fx, fy) in faces.clone() {
            if level.cell_at(fx, fy).is_none() {
                return Err(EngineError::PlacementOverHole);
            }

            if level.face_at(fx, fy).is_some() {
                return Err(EngineError::PlacementCollision);
            }
        }

        let mut deleted = false;
        for (f, fx, fy) in faces {
            let mut try_delete = |dx, dy| {
                let f1x = fx.checked_add_signed(dx).unwrap_or(w);
                let f1y = fy.checked_add_signed(dy).unwrap_or(h);
                if let Some((f1, p1)) = level.face_at(f1x, f1y) {
                    if f1.symbol == f.symbol {
                        let index = level.polyominoes.iter().position(|p| ptr::eq(p, p1));
                        level.polyominoes.remove(index.unwrap());
                        deleted = true;
                    }
                }
            };

            try_delete(-1, 0);
            try_delete(1, 0);
            try_delete(0, -1);
            try_delete(0, 1);
        }

        if !deleted {
            level.polyominoes.push(polyomino);
        }

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
