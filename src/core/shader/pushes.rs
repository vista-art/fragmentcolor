//! Values another object keeps in step on a shader,
//! such as a camera's matrix, and the refusals `Renderer::render` reports.

use super::{ShaderObject, UniformInput};
use crate::ShaderError;

impl ShaderObject {
    /// Writes `numbers` under `key` for an object that keeps the key in step.
    /// A key the shader doesn't declare is skipped, and a value the key can't hold
    /// stays for `check_pushes` to report until a value of the key lands. The check
    /// waits for a write on another thread, so a busy storage can't hide a refusal.
    pub(crate) fn push(&self, key: &str, numbers: &[f32]) {
        let input = UniformInput::from(numbers);
        let result = self
            .check_input(key, &input)
            .and_then(|()| self.set_input(key, input.clone()));
        if let Err(error) = result
            && !error.is_missing()
        {
            self.refused.write().insert(key.to_string(), input);
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

    // Story: a `Shader::set` that fits the key ends a refusal no later push brings,
    // as when the Scene that pushed the value lets the pass go.
    #[test]
    fn a_set_that_fits_ends_the_refusal() {
        let shader = half_ambient();
        shader.object.push("lights.ambient", &[70000.0, 0.0, 0.0]);

        shader.set("lights.ambient", [0.5, 0.25, 1.0]).expect("set");

        assert!(shader.object.check_pushes().is_ok());
    }

    // Story: a push to a key the shader doesn't declare leaves nothing to report.
    #[test]
    fn a_push_to_an_undeclared_key_stays_silent() {
        let shader = half_ambient();

        shader.object.push("camera.position", &[1e9, 0.0, 0.0]);

        assert!(shader.object.check_pushes().is_ok());
    }

    // Story: a shader whose `camera` is a plain matrix declares no `camera.view_proj`,
    // so the Camera's push skips it and the matrix the user set stays.
    #[test]
    fn a_push_past_a_plain_root_stays_silent() {
        let shader = Shader::new(
            "
@group(0) @binding(0) var<uniform> camera: mat4x4<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return camera * vec4<f32>(f32(i), 0.0, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
  return vec4<f32>(1.0);
}",
        )
        .expect("shader");
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        shader.set("camera", identity).expect("matrix");

        shader.object.push("camera.view_proj", &[2.0; 16]);
        shader.object.push("camera.position", &[1.0, 2.0, 3.0]);

        assert!(shader.object.check_pushes().is_ok());
        let camera: [[f32; 4]; 4] = shader.get("camera").expect("camera");
        assert_eq!(camera, identity);
    }
}
