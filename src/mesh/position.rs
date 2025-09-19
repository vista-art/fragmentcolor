#[derive(Clone, Debug, PartialEq)]
pub enum Position {
    Pos2([f32; 2]),
    Pos3([f32; 3]),
}

crate::impl_from_into_with_refs!(
    Position,
    (u32, u32),
    |p: Position| match p {
        Position::Pos2(a) => (a[0] as u32, a[1] as u32),
        Position::Pos3(a) => (a[0] as u32, a[1] as u32),
    },
    |t: (u32, u32)| Position::Pos2([t.0 as f32, t.1 as f32])
);

crate::impl_from_into_with_refs!(
    Position,
    (u32, u32, u32),
    |p: Position| match p {
        Position::Pos2(a) => (a[0] as u32, a[1] as u32, 1),
        Position::Pos3(a) => (a[0] as u32, a[1] as u32, a[2] as u32),
    },
    |t: (u32, u32, u32)| Position::Pos3([t.0 as f32, t.1 as f32, t.2 as f32])
);

crate::impl_from_into_with_refs!(
    Position,
    [u32; 2],
    |p: Position| match p {
        Position::Pos2(a) => [a[0] as u32, a[1] as u32],
        Position::Pos3(a) => [a[0] as u32, a[1] as u32],
    },
    |a: [u32; 2]| Position::Pos2([a[0] as f32, a[1] as f32])
);

crate::impl_from_into_with_refs!(
    Position,
    [u32; 3],
    |p: Position| match p {
        Position::Pos2(a) => [a[0] as u32, a[1] as u32, 1],
        Position::Pos3(a) => [a[0] as u32, a[1] as u32, a[2] as u32],
    },
    |a: [u32; 3]| Position::Pos3([a[0] as f32, a[1] as f32, a[2] as f32])
);

crate::impl_from_into_with_refs!(
    Position,
    [f32; 2],
    |p: Position| match p {
        Position::Pos2(a) => a,
        Position::Pos3(a) => [a[0], a[1]],
    },
    |a: [f32; 2]| Position::Pos2(a)
);

crate::impl_from_into_with_refs!(
    Position,
    [f32; 3],
    |p: Position| match p {
        Position::Pos2(a) => [a[0], a[1], 0.0],
        Position::Pos3(a) => a,
    },
    |a: [f32; 3]| Position::Pos3(a)
);

crate::impl_from_into_with_refs!(
    Position,
    (f32, f32),
    |p: Position| match p {
        Position::Pos2(a) => (a[0], a[1]),
        Position::Pos3(a) => (a[0], a[1]),
    },
    |t: (f32, f32)| Position::Pos2([t.0, t.1])
);

crate::impl_from_into_with_refs!(
    Position,
    (f32, f32, f32),
    |p: Position| match p {
        Position::Pos2(a) => (a[0], a[1], 0.0),
        Position::Pos3(a) => (a[0], a[1], a[2]),
    },
    |t: (f32, f32, f32)| Position::Pos3([t.0, t.1, t.2])
);
