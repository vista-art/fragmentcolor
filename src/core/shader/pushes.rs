//! Values another object keeps in step on a shader,
//! such as a camera's matrix, and the refusals `Renderer::render` reports.

use super::{ShaderObject, UniformInput};
use crate::ShaderError;

impl ShaderObject {
    /// Writes `numbers` under `key` for an object that keeps the key in step.
    /// A key the shader doesn't declare is skipped, and a value the key can't hold
    /// stays for `check_pushes` to report until a later push of the key lands.
    pub(crate) fn push(&self, key: &str, numbers: &[f32]) {
        let input = UniformInput::from(numbers);
        match self.set_input(key, input.clone()) {
            Ok(()) => {
                self.refused.write().remove(key);
            }
            Err(error) if error.is_missing() => {}
            Err(_) => {
                self.refused.write().insert(key.to_string(), input);
            }
        }
    }

    /// The first error of the values `push` kept, each shaped again to its key,
    /// so every render reports a camera or ambient value the shader can't hold.
    pub(crate) fn check_pushes(&self) -> Result<(), ShaderError> {
        let refused = self.refused.read();
        if refused.is_empty() {
            return Ok(());
        }
        let storage = self.storage.read();
        for (key, input) in refused.iter() {
            self.shape_input(&storage, key, input)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Shader;

    /// A fullscreen shader whose `lights` uniform holds an `f16` ambient term.
    fn half_ambient() -> Shader {
        Shader::new(
            "enable f16;
struct Lights { ambient: vec3<f16> }
@group(0) @binding(0) var<uniform> lights: Lights;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(f32(i), 0.0, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
  return vec4<f32>(vec3<f32>(lights.ambient), 1.0);
}",
        )
        .expect("shader")
    }

    // Story: a pushed value the key can't hold stays as the shader's refusal, which each
    // check reports, and a later push of the key that fits takes it off.
    #[test]
    fn a_refused_push_stays_until_one_fits() {
        let shader = half_ambient();

        shader.object.push("lights.ambient", &[70000.0, 0.0, 0.0]);
        let first = shader.object.check_pushes();
        let second = shader.object.check_pushes();
        shader.object.push("lights.ambient", &[0.5, 0.25, 1.0]);
        let fitted = shader.object.check_pushes();

        let message = first.expect_err("past the f16 range").to_string();
        assert!(
            message.contains("lights.ambient: expected vec3<f16>, got 70000"),
            "{message}"
        );
        assert!(second.is_err(), "the refusal left after one check");
        assert!(fitted.is_ok(), "{fitted:?}");
        let ambient: [f32; 3] = shader.get("lights.ambient").expect("ambient");
        assert_eq!(ambient, [0.5, 0.25, 1.0]);
    }

    // Story: a push to a key the shader doesn't declare leaves nothing to report.
    #[test]
    fn a_push_to_an_undeclared_key_stays_silent() {
        let shader = half_ambient();

        shader.object.push("camera.position", &[1e9, 0.0, 0.0]);

        assert!(shader.object.check_pushes().is_ok());
    }
}
