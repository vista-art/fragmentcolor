use crate::scene::{macros::api_object, Object};

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty;

api_object!(Empty);

impl Empty {
    pub fn new() -> Object<Self> {
        Object::new(Self)
    }
}
