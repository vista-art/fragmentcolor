use crate::{Pass, PassObject};

pub trait Renderable {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject>;
}

impl Renderable for &[Pass] {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iter().map(|p| p.object.as_ref())
    }
}

impl Renderable for Vec<Pass> {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iter().map(|p| p.object.as_ref())
    }
}

impl Renderable for &[&Pass] {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iter().map(|p| p.object.as_ref())
    }
}

impl Renderable for Vec<&Pass> {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iter().map(|p| p.object.as_ref())
    }
}

impl Renderable for &[PassObject] {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iter()
    }
}

impl Renderable for Vec<PassObject> {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iter()
    }
}

impl Renderable for &[&PassObject] {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iter().copied()
    }
}

impl Renderable for Vec<&PassObject> {
    fn passes(&self) -> impl IntoIterator<Item = &PassObject> {
        self.iter().copied()
    }
}
