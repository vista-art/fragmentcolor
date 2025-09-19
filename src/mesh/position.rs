#[derive(Clone, Debug, PartialEq)]
pub enum Position {
    Pos2([f32; 2]),
    Pos3([f32; 3]),
}

impl From<(u32, u32)> for Position {
    fn from(value: (u32, u32)) -> Self {
        Self::Pos2([value.0 as f32, value.1 as f32])
    }
}

impl From<Position> for (u32, u32) {
    fn from(position: Position) -> Self {
        match position {
            Position::Pos2(arr) => (arr[0] as u32, arr[1] as u32),
            Position::Pos3(arr) => (arr[0] as u32, arr[1] as u32),
        }
    }
}

impl From<&(u32, u32)> for Position {
    fn from(value: &(u32, u32)) -> Self {
        Self::Pos2([value.0 as f32, value.1 as f32])
    }
}

impl From<&Position> for (u32, u32) {
    fn from(position: &Position) -> Self {
        match position {
            Position::Pos2(arr) => (arr[0] as u32, arr[1] as u32),
            Position::Pos3(arr) => (arr[0] as u32, arr[1] as u32),
        }
    }
}

impl From<(u32, u32, u32)> for Position {
    fn from(value: (u32, u32, u32)) -> Self {
        Self::Pos3([value.0 as f32, value.1 as f32, value.2 as f32])
    }
}

impl From<Position> for (u32, u32, u32) {
    fn from(position: Position) -> Self {
        match position {
            Position::Pos2(arr) => (arr[0] as u32, arr[1] as u32, 1),
            Position::Pos3(arr) => (arr[0] as u32, arr[1] as u32, arr[2] as u32),
        }
    }
}

impl From<&(u32, u32, u32)> for Position {
    fn from(value: &(u32, u32, u32)) -> Self {
        Self::Pos3([value.0 as f32, value.1 as f32, value.2 as f32])
    }
}

impl From<&Position> for (u32, u32, u32) {
    fn from(position: &Position) -> Self {
        match position {
            Position::Pos2(arr) => (arr[0] as u32, arr[1] as u32, 1),
            Position::Pos3(arr) => (arr[0] as u32, arr[1] as u32, arr[2] as u32),
        }
    }
}

impl From<[u32; 2]> for Position {
    fn from(value: [u32; 2]) -> Self {
        Self::Pos2(value.map(|v| v as f32))
    }
}

impl From<Position> for [u32; 2] {
    fn from(position: Position) -> Self {
        match position {
            Position::Pos2(arr) => [arr[0] as u32, arr[1] as u32],
            Position::Pos3(arr) => [arr[0] as u32, arr[1] as u32],
        }
    }
}

impl From<&[u32; 2]> for Position {
    fn from(value: &[u32; 2]) -> Self {
        Self::Pos2(value.map(|v| v as f32))
    }
}

impl From<&Position> for [u32; 2] {
    fn from(position: &Position) -> Self {
        match position {
            Position::Pos2(arr) => [arr[0] as u32, arr[1] as u32],
            Position::Pos3(arr) => [arr[0] as u32, arr[1] as u32],
        }
    }
}

impl From<[u32; 3]> for Position {
    fn from(value: [u32; 3]) -> Self {
        Self::Pos3(value.map(|v| v as f32))
    }
}

impl From<Position> for [u32; 3] {
    fn from(position: Position) -> Self {
        match position {
            Position::Pos2(arr) => [arr[0] as u32, arr[1] as u32, 1],
            Position::Pos3(arr) => [arr[0] as u32, arr[1] as u32, arr[2] as u32],
        }
    }
}

impl From<&[u32; 3]> for Position {
    fn from(value: &[u32; 3]) -> Self {
        Self::Pos3(value.map(|v| v as f32))
    }
}

impl From<&Position> for [u32; 3] {
    fn from(position: &Position) -> Self {
        match position {
            Position::Pos2(arr) => [arr[0] as u32, arr[1] as u32, 1],
            Position::Pos3(arr) => [arr[0] as u32, arr[1] as u32, arr[2] as u32],
        }
    }
}
