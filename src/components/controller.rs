use crate::transform::Transform;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};

#[derive(Debug)]
pub struct Controller {
    pub speed: f32,
    pub is_forward_pressed: bool,
    pub is_backward_pressed: bool,
    pub is_left_pressed: bool,
    pub is_right_pressed: bool,
}

impl Controller {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput {
            event: KeyEvent {
                state, logical_key, ..
            },
            ..
        } = event
        {
            let is_pressed = state == &ElementState::Pressed;

            match logical_key {
                Key::Character(c) => match c.as_str() {
                    "w" => {
                        self.is_forward_pressed = is_pressed;
                    }
                    "s" => {
                        self.is_backward_pressed = is_pressed;
                    }
                    "a" => {
                        self.is_left_pressed = is_pressed;
                    }
                    "d" => {
                        self.is_right_pressed = is_pressed;
                    }
                    _ => {}
                },
                Key::Named(NamedKey::ArrowUp) => {
                    self.is_forward_pressed = is_pressed;
                }
                Key::Named(NamedKey::ArrowDown) => {
                    self.is_backward_pressed = is_pressed;
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    self.is_left_pressed = is_pressed;
                }
                Key::Named(NamedKey::ArrowRight) => {
                    self.is_right_pressed = is_pressed;
                }
                _ => {}
            }
        }
    }

    pub fn update_transform(&self, _transform: &mut Transform) {
        todo!();

        // @TODO uncomment and implement.
        //       This came from the old code that uses CGMath.
        //       I have to convert it to glam and make it generic
        //       (i.e.) not specific to a camera

        // let forward = transform.target - transform.eye;
        // let forward_norm = forward.normalize();
        // let forward_mag = forward.magnitude();

        // // Prevents glitching when transform gets too close to the
        // // center of the scene.
        // if self.is_forward_pressed && forward_mag > self.speed {
        //     transform.eye += forward_norm * self.speed;
        // }
        // if self.is_backward_pressed {
        //     transform.eye -= forward_norm * self.speed;
        // }

        // let right = forward_norm.cross(transform.up);

        // // Redo radius calc in case the fowrard/backward is pressed.
        // let forward = transform.target - transform.eye;
        // let forward_mag = forward.magnitude();

        // if self.is_right_pressed {
        //     // Rescale the distance between the target and eye so
        //     // that it doesn't change. The eye therefore still
        //     // lies on the circle made by the target and eye.
        //     transform.eye = transform.target - (forward + right * self.speed).normalize() * forward_mag;
        // }
        // if self.is_left_pressed {
        //     transform.eye = transform.target - (forward - right * self.speed).normalize() * forward_mag;
        // }
    }
}
