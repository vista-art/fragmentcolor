use crate::scene::macros::api_object;
use crate::{components::Color, transform::TransformId};
use crate::{Border, Bounds, Object, Pixel, Quad, Renderable2D, SceneObject, ShapeFlag};
use derive_setters::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Shape {
    pub transform_id: TransformId,
}

api_object!(Shape);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShapeType {
    None = 0,
    Circle,
    Rectangle,
    Line,
}

impl Default for ShapeType {
    fn default() -> Self {
        Self::Circle
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Setters)]
#[setters(prefix = "set_")]
pub struct ShapeOptions {
    pub bounds: Quad,
    pub color: Color,
    pub border: f32,
}

impl Shape {
    pub fn new(options: &ShapeOptions, shape_type: ShapeType) -> Object<Self> {
        let mut shape = Object::new(Shape::default());

        let components = Renderable2D {
            transform: shape.transform_id(),
            image: None,
            bounds: Bounds(options.bounds),
            color: options.color,
            border: Border(options.border),
            sdf_flags: match shape_type {
                ShapeType::None => ShapeFlag(0.0),
                ShapeType::Circle => ShapeFlag(1.0),
                ShapeType::Rectangle => ShapeFlag(2.0),
                ShapeType::Line => ShapeFlag(3.0),
            },
        };

        shape.add_components(components);

        shape
    }
}

impl Object<Shape> {
    pub(crate) fn bounds(&self) -> Quad {
        if let Some(bounds) = self.read_component::<Bounds>() {
            bounds.0
        } else {
            Quad::default()
        }
    }

    pub fn width(&self) -> f32 {
        self.bounds().width() as f32
    }

    pub fn set_width(&mut self, width: f32) -> &mut Self {
        self.update_component(Bounds(Quad::from_size(width as u32, self.height() as u32)));
        self
    }

    pub fn height(&self) -> f32 {
        self.bounds().height() as f32
    }

    pub fn set_height(&mut self, height: f32) -> &mut Self {
        self.update_component(Bounds(Quad::from_size(self.width() as u32, height as u32)));
        self
    }

    pub fn from(&self) -> Pixel {
        let bounds = self.bounds();
        Pixel {
            x: bounds.min_x as u16,
            y: bounds.min_y as u16,
        }
    }

    pub fn to(&self) -> Pixel {
        let bounds = self.bounds();
        Pixel {
            x: bounds.max_x as u16,
            y: bounds.max_y as u16,
        }
    }

    pub fn set_from(&mut self, from: Pixel) -> &mut Self {
        let to = self.to();
        let bounds = Bounds(Quad::from_tuples(
            (from.x as u32, from.y as u32),
            (to.x as u32, to.y as u32),
        ));
        self.update_component(bounds);
        self
    }

    pub fn set_to(&mut self, to: Pixel) -> &mut Self {
        let from = self.from();
        let bounds = Bounds(Quad::from_tuples(
            (from.x as u32, from.y as u32),
            (to.x as u32, to.y as u32),
        ));
        self.update_component(bounds);
        self
    }

    pub fn color(&self) -> Color {
        if let Some(color) = self.read_component::<Color>() {
            color
        } else {
            Color::default()
        }
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.update_component(color);
        self
    }

    // NOTE: Radius is a computed property based on bounds and border
    pub fn radius(&self) -> f32 {
        let bounds = self.bounds();
        let border = self.border();
        bounds.inbound_radius() - border
    }

    // Updating Radius will change the bounds
    pub fn set_radius(&mut self, radius: f32) -> &mut Self {
        let border = self.border();
        let bounds = Bounds(Quad::from_inbound_radius(radius + border));
        self.update_component(bounds);
        self
    }

    pub fn border(&self) -> f32 {
        if let Some(border) = self.read_component::<Border>() {
            border.0
        } else {
            0.0
        }
    }

    // Updating Border will change the bounds
    pub fn set_border(&mut self, border: f32) -> &mut Self {
        let radius = self.radius();
        let bounds = Bounds(Quad::from_inbound_radius(radius + border));
        self.update_components((bounds, Border(border)));
        self
    }

    pub fn thickness(&self) -> f32 {
        self.border()
    }

    pub fn set_thickness(&mut self, thickness: f32) -> &mut Self {
        self.set_border(thickness)
    }

    pub(crate) fn sdf_flags(&self) -> f32 {
        if let Some(sdf_flags) = self.read_component::<ShapeType>() {
            sdf_flags as u8 as f32
        } else {
            0.0
        }
    }

    pub fn shape_type(&self) -> ShapeType {
        match self.sdf_flags() as u8 {
            0 => ShapeType::Circle,
            1 => ShapeType::Rectangle,
            2 => ShapeType::Line,
            _ => ShapeType::Circle,
        }
    }

    pub fn set_shape_type(&mut self, shape: ShapeType) -> &mut Self {
        self.update_component(ShapeFlag(shape as u8 as f32));
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Setters)]
#[setters(prefix = "set_")]
pub struct CircleOptions {
    pub radius: f32,
    pub color: Color,
    pub border: f32,
}

impl Default for CircleOptions {
    fn default() -> Self {
        Self {
            color: Color(0xff000088),
            border: 0.0,
            radius: 0.0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Circle;
impl Circle {
    pub fn new(options: CircleOptions) -> Object<Shape> {
        let options = ShapeOptions {
            bounds: Quad::from_inbound_radius(options.radius + options.border),
            color: options.color,
            border: options.border,
        };
        Shape::new(&options, ShapeType::Circle)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Rectangle;
impl Rectangle {
    pub fn new(options: ShapeOptions) -> Object<Shape> {
        Shape::new(&options, ShapeType::Rectangle)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Square;
impl Square {
    pub fn new(size: u32) -> Object<Shape> {
        Rectangle::new(ShapeOptions {
            bounds: Quad::from_size(size, size),
            ..Default::default()
        })
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Point;
impl Point {
    pub fn new() -> Object<Shape> {
        Circle::new(CircleOptions {
            radius: 0.5,
            ..Default::default()
        })
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Line;
impl Line {
    pub fn new(from: Pixel, to: Pixel) -> Object<Shape> {
        let bounds = Quad::from_tuples((from.x as u32, from.y as u32), (to.x as u32, to.y as u32));
        Shape::new(
            &ShapeOptions {
                bounds,
                ..Default::default()
            },
            ShapeType::Line,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn test_width_height_set_get() {
        let mut shape = Circle::new(CircleOptions::default());
        shape.set_width(50.0);
        shape.set_height(100.0);

        assert_eq!(shape.width(), 50.0);
        assert_eq!(shape.height(), 100.0);
    }

    #[test]
    fn test_color_set_get() {
        let mut shape = Circle::new(CircleOptions::default());
        shape.set_color(Color(0xff000000));

        assert_eq!(shape.color(), Color(0xff000000));
    }
}
