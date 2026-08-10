use crate::level::{FaceType, Level, Polyomino};

pub enum EngineError {
    PolyominoNotFound,
    PlacementOutOfBounds,
    PlacementCollision,
}

pub struct Engine {
    original_level: Level,
    level: Level,
}

impl Engine {
    pub fn new(level: Level) -> Self {
        Engine {
            original_level: level.clone(),
            level,
        }
    }

    pub fn undo(&mut self) {}

    pub fn redo(&mut self) {}

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
                (face.dx, -face.dy)
            } else {
                (-face.dx, face.dy)
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
        // self.process_face_interactions(self.level.polyominoes.len())
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

//     fn process_face_interactions(&mut self, polyomino_index: usize) -> Result<(), EngineError> {
//         let p0 = self.get_polyomino(polyomino_index)?;
//
//         for f0 in p.faces.iter() {
//             let x = p.x.wrapping_add_signed(f.dx);
//             let y = p.y.wrapping_add_signed(f.dy);
//
//             let mut interact = |x1: usize, y1: usize| -> Option<()> {
//                 if let Some((f1, p1)) = self.level.face_at(x1, y1) {
//                     match f.symbol {
//                         FaceType::Mirror => {}
//                         FaceType::Boom => {}
//                         _ => {
//                             if f.symbol as u8 == f1.symbol as u8 {
//                                 self.damage_polyomino(hand_index);
//                             }
//                         }
//                     }
//                 }
//                 None
//             };
//
//             interact(x.wrapping_sub(1), y);
//             interact(x.wrapping_add(1), y);
//             interact(x, y.wrapping_sub(1));
//             interact(x, y.wrapping_add(1));
//         }
//     }
}
