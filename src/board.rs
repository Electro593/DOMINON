use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    iter::repeat_with,
    ops::{Index, IndexMut},
};

use crate::level::{Level, RawCellType, RawFaceType, RawPolyomino};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum CellType {
    None,
    Basic,
}

impl From<RawCellType> for CellType {
    fn from(value: RawCellType) -> Self {
        match value {
            RawCellType::None => Self::None,
            RawCellType::Basic => Self::Basic,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum FaceType {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
}

impl TryFrom<RawFaceType> for FaceType {
    type Error = &'static str;

    fn try_from(value: RawFaceType) -> Result<Self, Self::Error> {
        match value {
            RawFaceType::Zero => Ok(Self::Zero),
            RawFaceType::One => Ok(Self::One),
            RawFaceType::Two => Ok(Self::Two),
            RawFaceType::Three => Ok(Self::Three),
            RawFaceType::Four => Ok(Self::Four),
            RawFaceType::Five => Ok(Self::Five),
            RawFaceType::Six => Ok(Self::Six),
            RawFaceType::Seven => Ok(Self::Seven),
            RawFaceType::Eight => Ok(Self::Eight),
            RawFaceType::Nine => Ok(Self::Nine),
            RawFaceType::Ten => Ok(Self::Ten),
            RawFaceType::Eleven => Ok(Self::Eleven),
            RawFaceType::Twelve => Ok(Self::Twelve),
            _ => Err("Inapplicable raw face type"),
        }
    }
}

#[derive(Debug)]
pub enum BoardError {
    CannotUndo,
    CannotRedo,
    PolyominoNotFound,
    PlacementOutOfBounds,
    PlacementOverHole,
    PlacementCollision,
}

impl Error for BoardError {}

impl Display for BoardError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PolyominoRef {
    index: u8,
}

impl PolyominoRef {
    const NONE: Self = Self { index: u8::MAX };

    fn is_none(&self) -> bool {
        self == &Self::NONE
    }

    fn is_some(&self) -> bool {
        !self.is_none()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FaceRef {
    index: u8,
}

impl FaceRef {
    const NONE: Self = Self { index: u8::MAX };

    fn is_none(&self) -> bool {
        self == &Self::NONE
    }

    fn is_some(&self) -> bool {
        !self.is_none()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct GridPos {
    x: u8,
    y: u8,
}

impl GridPos {
    const HAND: Self = Self { x: u8::MAX, y: 0 };
    const NONE: Self = Self {
        x: u8::MAX,
        y: u8::MAX,
    };

    fn in_hand(&self) -> bool {
        self == &Self::HAND
    }

    fn in_grid(&self) -> bool {
        self.x < u8::MAX && self.y < u8::MAX
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

impl Rotation {
    fn transform(&self, x: i8, y: i8) -> (i8, i8) {
        match self {
            Self::None => (x, y),
            Self::Clockwise90 => (y, -x),
            Self::Clockwise180 => (-x, -y),
            Self::Clockwise270 => (-y, x),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BoardDelta {
    Rotate(PolyominoRef, Rotation),
    Place(PolyominoRef, GridPos),
    Group(Vec<Self>),
}

#[derive(Clone, Copy, Debug)]
pub struct Polyomino {
    pos: GridPos,
    dir: Rotation,
    first: FaceRef,
    n: u8,
}

impl Polyomino {
    fn checked_face_pos(&self, face: &Face) -> Option<GridPos> {
        let (dx, dy) = self.dir.transform(face.dx, face.dy);
        let x = self.pos.x.checked_add_signed(dx)?;
        let y = self.pos.y.checked_add_signed(dy)?;
        Some(GridPos { x, y })
    }

    fn wrapping_face_pos(&self, face: &Face) -> GridPos {
        let (dx, dy) = self.dir.transform(face.dx, face.dy);
        let x = self.pos.x.wrapping_add_signed(dx);
        let y = self.pos.y.wrapping_add_signed(dy);
        GridPos { x, y }
    }
}

#[derive(Clone, Copy, Debug)]
struct Face {
    parent: PolyominoRef,
    ty: FaceType,
    dx: i8,
    dy: i8,
}

#[derive(Clone, Debug)]
struct Faces {
    i: u8,
    e: u8,
}

impl<'f> Iterator for Faces {
    type Item = FaceRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.e || self.i == FaceRef::NONE.index {
            return None;
        }

        let face_ref = FaceRef { index: self.i };
        self.i += 1;
        Some(face_ref)
    }
}

#[derive(Clone, Debug)]
pub struct BoardPieces {
    faces: Vec<Face>,
    polyominoes: Vec<Polyomino>,
}

impl BoardPieces {
    fn face(&self, face_ref: FaceRef) -> Option<&Face> {
        face_ref
            .is_some()
            .then(|| &self.faces[face_ref.index as usize])
    }

    fn polyomino(&self, polyomino_ref: PolyominoRef) -> Option<&Polyomino> {
        polyomino_ref
            .is_some()
            .then(|| &self.polyominoes[polyomino_ref.index as usize])
    }

    fn polyomino_mut(&mut self, polyomino_ref: PolyominoRef) -> Option<&mut Polyomino> {
        polyomino_ref
            .is_some()
            .then(|| &mut self.polyominoes[polyomino_ref.index as usize])
    }

    fn faces_of(&self, polyomino_ref: PolyominoRef) -> Faces {
        self.polyomino(polyomino_ref)
            .map(|polyomino| Faces {
                i: polyomino.first.index,
                e: polyomino.first.index + polyomino.n,
            })
            .unwrap_or(
                const {
                    Faces {
                        i: FaceRef::NONE.index,
                        e: FaceRef::NONE.index,
                    }
                },
            )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub base: CellType,
    pub face: FaceRef,
}

#[derive(Clone, Debug)]
pub struct BoardGrid {
    width: u8,
    height: u8,
    cells: Vec<Cell>,
}

impl BoardGrid {
    fn index_of(pos: GridPos, width: u8) -> usize {
        pos.y as usize * width as usize + pos.x as usize
    }

    fn checked_face_pos(&self, polyomino: &Polyomino, face: &Face) -> Option<GridPos> {
        let pos = polyomino.checked_face_pos(face)?;
        (pos.x < self.width && pos.y < self.height).then_some(pos)
    }

    fn remove_polyomino(&mut self, pieces: &BoardPieces, polyomino_ref: PolyominoRef) {
        pieces.polyomino(polyomino_ref).map(|polyomino| {
            if polyomino.pos.in_grid() {
                for face_ref in pieces.faces_of(polyomino_ref) {
                    pieces.face(face_ref).map(|face| {
                        let p = polyomino.wrapping_face_pos(face);
                        self[p].face = FaceRef::NONE;
                    });
                }
            }
        });
    }

    fn place_polyomino(&mut self, pieces: &BoardPieces, polyomino_ref: PolyominoRef) {
        pieces.polyomino(polyomino_ref).map(|polyomino| {
            if polyomino.pos.in_grid() {
                for face_ref in pieces.faces_of(polyomino_ref) {
                    pieces.face(face_ref).map(|face| {
                        let p = polyomino.wrapping_face_pos(face);
                        self[p].face = face_ref;
                    });
                }
            }
        });
    }
}

impl Index<GridPos> for BoardGrid {
    type Output = Cell;

    fn index(&self, index: GridPos) -> &Self::Output {
        &self.cells[Self::index_of(index, self.width)]
    }
}

impl IndexMut<GridPos> for BoardGrid {
    fn index_mut(&mut self, index: GridPos) -> &mut Cell {
        &mut self.cells[Self::index_of(index, self.width)]
    }
}

#[derive(Clone, Debug)]
pub struct BoardState {
    pub pieces: BoardPieces,
    pub grid: BoardGrid,
}

impl BoardState {
    pub fn are_all_destroyed(&self) -> bool {
        self.pieces
            .polyominoes
            .iter()
            .filter(|p| p.pos != GridPos::NONE)
            .count()
            == 0
    }

    pub fn possible_moves(&self, limit: usize) -> Vec<BoardDelta> {
        let rotations = const {
            [
                Rotation::None,
                Rotation::Clockwise90,
                Rotation::Clockwise180,
                Rotation::Clockwise270,
            ]
        };

        let flatten = |pref, pos, pdir, dir| {
            let dpos = BoardDelta::Place(pref, pos);
            let ddir = BoardDelta::Rotate(pref, dir);
            if pdir != dir {
                BoardDelta::Group(vec![ddir, dpos])
            } else {
                dpos
            }
        };

        self.pieces
            .polyominoes
            .iter()
            .enumerate()
            .filter(|(_, p)| p.pos == GridPos::HAND)
            .flat_map(|(i, p)| {
                let polyomino_ref = PolyominoRef { index: i as u8 };
                (0..self.grid.height).flat_map(move |y| {
                    (0..self.grid.width).flat_map(move |x| {
                        let pos = GridPos { x, y };
                        rotations.into_iter().filter_map(move |dir| {
                            self.can_place(polyomino_ref, Some(pos), Some(dir))
                                .map(|_| flatten(polyomino_ref, pos, p.dir, dir))
                                .ok()
                        })
                    })
                })
            })
            .take(limit)
            .collect()
    }

    pub fn can_place(
        &self,
        polyomino_ref: PolyominoRef,
        pos: Option<GridPos>,
        dir: Option<Rotation>,
    ) -> Result<(), BoardError> {
        let polyomino = self
            .pieces
            .polyomino(polyomino_ref)
            .map(|original| Polyomino {
                pos: pos.unwrap_or(original.pos),
                dir: dir.unwrap_or(original.dir),
                ..*original
            })
            .ok_or(BoardError::PolyominoNotFound)?;

        if !polyomino.pos.in_grid() {
            return Ok(());
        }

        for i in 0..polyomino.n {
            let face_ref = FaceRef {
                index: polyomino.first.index + i,
            };
            let face = self.pieces.face(face_ref);
            if face.is_none() {
                continue;
            }

            let face_pos = self.grid.checked_face_pos(&polyomino, face.unwrap());
            if face_pos.is_none() {
                return Err(BoardError::PlacementOutOfBounds);
            }

            let cell = &self.grid[face_pos.unwrap()];
            if let CellType::None = cell.base {
                return Err(BoardError::PlacementOverHole);
            }

            let face_at_pos = self.pieces.face(cell.face);
            if face_at_pos.is_some() && face.unwrap().parent != face_at_pos.unwrap().parent {
                return Err(BoardError::PlacementCollision);
            }
        }

        return Ok(());
    }

    fn apply_delta(&mut self, delta: &BoardDelta) -> Result<BoardDelta, BoardError> {
        match delta {
            BoardDelta::Rotate(polyomino_ref, dir) => {
                self.can_place(*polyomino_ref, None, Some(*dir))?;
                self.grid.remove_polyomino(&self.pieces, *polyomino_ref);

                let polyomino = self.pieces.polyomino_mut(*polyomino_ref).unwrap();
                let old_dir = polyomino.dir;
                polyomino.dir = *dir;
                self.grid.place_polyomino(&self.pieces, *polyomino_ref);

                return Ok(BoardDelta::Rotate(*polyomino_ref, old_dir));
            }
            BoardDelta::Place(polyomino_ref, pos) => {
                self.can_place(*polyomino_ref, Some(*pos), None)?;
                self.grid.remove_polyomino(&self.pieces, *polyomino_ref);

                let polyomino = self.pieces.polyomino_mut(*polyomino_ref).unwrap();
                let old_pos = polyomino.pos;
                polyomino.pos = *pos;
                self.grid.place_polyomino(&self.pieces, *polyomino_ref);

                return Ok(BoardDelta::Place(*polyomino_ref, old_pos));
            }
            BoardDelta::Group(group) => {
                let mut deltas = group
                    .iter()
                    .map(|delta| self.apply_delta(delta))
                    .collect::<Result<Vec<BoardDelta>, BoardError>>()?;

                deltas.reverse();

                return Ok(BoardDelta::Group(deltas));
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    pub state: BoardState,
    past: Vec<BoardDelta>,
    future: Vec<BoardDelta>,
}

impl Board {
    pub fn undo(&mut self) -> Result<BoardDelta, BoardError> {
        let delta = self.past.last().ok_or(BoardError::CannotUndo)?;
        let result = self.state.apply_delta(delta)?;
        self.future.push(result);
        Ok(self.past.pop().unwrap())
    }

    pub fn redo(&mut self) -> Result<BoardDelta, BoardError> {
        let delta = self.future.last().ok_or(BoardError::CannotRedo)?;
        let result = self.state.apply_delta(delta)?;
        self.past.push(result);
        Ok(self.future.pop().unwrap())
    }

    fn push(&mut self, delta: BoardDelta) -> Result<BoardDelta, BoardError> {
        let inverse = self.state.apply_delta(&delta)?;
        self.past.push(inverse);
        self.future.clear();
        Ok(delta)
    }

    pub fn reset(&mut self) -> Result<BoardDelta, BoardError> {
        if self.past.is_empty() {
            return Ok(BoardDelta::Group(vec![]));
        }

        self.push(BoardDelta::Group(
            self.past
                .iter()
                .map(|d| d.clone())
                .rev()
                .collect::<Vec<BoardDelta>>(),
        ))
    }

    pub fn rotate(
        &mut self,
        polyomino_ref: PolyominoRef,
        dir: Rotation,
    ) -> Result<BoardDelta, BoardError> {
        self.push(BoardDelta::Rotate(polyomino_ref, dir))
    }

    pub fn place(
        &mut self,
        polyomino_ref: PolyominoRef,
        pos: GridPos,
    ) -> Result<BoardDelta, BoardError> {
        self.push(BoardDelta::Place(polyomino_ref, pos))
    }
}

impl TryFrom<Level> for Board {
    type Error = Box<dyn Error>;

    fn try_from(level: Level) -> Result<Self, Self::Error> {
        let width = level.cells.iter().map(|r| r.len()).max().unwrap_or(0);
        let height = level.cells.len();
        let cells = level
            .cells
            .iter()
            .flat_map(|row| {
                row.iter()
                    .chain(repeat_with(|| &RawCellType::None))
                    .take(width)
                    .map(|c| Cell {
                        base: (*c).into(),
                        face: FaceRef::NONE,
                    })
            })
            .collect();

        let mut grid = BoardGrid {
            width: width.try_into()?,
            height: height.try_into()?,
            cells,
        };

        let mut pieces = BoardPieces {
            polyominoes: vec![],
            faces: vec![],
        };

        let mut map_polyomino = |p: &RawPolyomino, in_hand| -> Result<(), Box<dyn Error>> {
            let first = FaceRef {
                index: pieces.faces.len().try_into()?,
            };
            let parent = PolyominoRef {
                index: pieces.polyominoes.len().try_into()?,
            };

            let pos = if in_hand {
                GridPos::HAND
            } else {
                GridPos {
                    x: p.x.try_into()?,
                    y: p.y.try_into()?,
                }
            };

            let polyomino = Polyomino {
                pos,
                dir: Rotation::None,
                first,
                n: p.faces.len().try_into()?,
            };

            for (face_index, face) in p.faces.iter().enumerate() {
                let fi: u8 = face_index.try_into()?;
                let face = Face {
                    parent,
                    ty: face.symbol.try_into()?,
                    dx: face.dx.try_into()?,
                    dy: face.dy.try_into()?,
                };

                grid.checked_face_pos(&polyomino, &face)
                    .map(|fp| {
                        grid[fp].face = FaceRef {
                            index: first.index.checked_add(fi)?,
                        };
                        Some(())
                    })
                    .ok_or("Face out of bounds")?;

                pieces.faces.push(face);
            }

            pieces.polyominoes.push(polyomino);
            Ok(())
        };

        for polyomino in level.polyominoes.iter() {
            map_polyomino(polyomino, false)?;
        }

        for polyomino in level.hand.iter() {
            map_polyomino(polyomino, true)?;
        }

        Ok(Board {
            state: BoardState { pieces, grid },
            past: vec![],
            future: vec![],
        })
    }
}
