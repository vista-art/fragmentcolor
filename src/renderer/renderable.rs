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

#[cfg(test)]
mod more_tests {
    use super::*;

    // Story: Containers backed by PassObject values expose references via Renderable.
    #[test]
    fn exposes_passobjects_direct_containers() {
        let p1 = PassObject::new("p1", crate::pass::PassType::Render);
        let p2 = PassObject::new("p2", crate::pass::PassType::Render);

        // &[PassObject]
        let arr = [p1, p2];
        let n = (&arr[..]).passes().into_iter().count();
        assert_eq!(n, 2);

        // Vec<PassObject>
        let v = vec![
            PassObject::new("p3", crate::pass::PassType::Render),
            PassObject::new("p4", crate::pass::PassType::Render),
        ];
        let n = v.passes().into_iter().count();
        assert_eq!(n, 2);

        // &[&PassObject]
        let pa = PassObject::new("p5", crate::pass::PassType::Render);
        let pb = PassObject::new("p6", crate::pass::PassType::Render);
        let refs = [&pa, &pb];
        let n = (&refs[..]).passes().into_iter().count();
        assert_eq!(n, 2);

        // Vec<&PassObject>
        let vrefs = vec![&pa, &pb];
        let n = vrefs.passes().into_iter().count();
        assert_eq!(n, 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Different containers should expose PassObject references consistently via Renderable.
    #[test]
    fn exposes_passes_from_various_containers() {
        // Arrange
        let p1 = Pass::new("p1");
        let p2 = Pass::new("p2");

        // Act / Assert: slice of Pass
        let arr = [p1.clone(), p2.clone()];
        let n = (&arr[..]).passes().into_iter().count();
        assert_eq!(n, 2);

        // Act / Assert: Vec<Pass>
        let v = vec![p1.clone(), p2.clone()];
        let n = v.passes().into_iter().count();
        assert_eq!(n, 2);

        // Act / Assert: Vec<&Pass>
        let v = vec![&p1, &p2];
        let n = v.passes().into_iter().count();
        assert_eq!(n, 2);
    }
}
