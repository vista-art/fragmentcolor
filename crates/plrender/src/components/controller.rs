use crate::scene::node::Node;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

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
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn update_position(&self, _node: &mut Node) {
        todo!();

        // @TODO uncomment and implement.
        //       This came from the old code that uses CGMatch.
        //       I have to convert it to glam and make it generic
        //       (i.e.) not specific to a camera

        // let forward = node.target - node.eye;
        // let forward_norm = forward.normalize();
        // let forward_mag = forward.magnitude();

        // // Prevents glitching when node gets too close to the
        // // center of the scene.
        // if self.is_forward_pressed && forward_mag > self.speed {
        //     node.eye += forward_norm * self.speed;
        // }
        // if self.is_backward_pressed {
        //     node.eye -= forward_norm * self.speed;
        // }

        // let right = forward_norm.cross(node.up);

        // // Redo radius calc in case the fowrard/backward is pressed.
        // let forward = node.target - node.eye;
        // let forward_mag = forward.magnitude();

        // if self.is_right_pressed {
        //     // Rescale the distance between the target and eye so
        //     // that it doesn't change. The eye therefore still
        //     // lies on the circle made by the target and eye.
        //     node.eye = node.target - (forward + right * self.speed).normalize() * forward_mag;
        // }
        // if self.is_left_pressed {
        //     node.eye = node.target - (forward - right * self.speed).normalize() * forward_mag;
        // }
    }
}
