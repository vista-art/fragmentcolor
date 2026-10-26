#![cfg(wasm)]

use crate::platforms::web::read_unsigned;
use crate::renderer::platform::web::RenderableList;
use crate::{Color, Pass, Shader, Texture, TextureTarget};
use lsp_doc::lsp_doc;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Pass {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/core/pass/new.md")]
    pub fn new_js(name: &str) -> Self {
        Self::new(name)
    }

    #[wasm_bindgen(js_name = "compute")]
    #[lsp_doc("docs/api/core/pass/compute.md")]
    pub fn compute_js(name: &str) -> Self {
        Self::compute(name)
    }

    #[wasm_bindgen(js_name = "fromShader")]
    #[lsp_doc("docs/api/core/pass/from_shader.md")]
    pub fn from_shader_js(name: &str, shader: &Shader) -> Self {
        Self::from_shader(name, shader)
    }

    #[wasm_bindgen(js_name = "require")]
    #[lsp_doc("docs/api/core/pass/require.md")]
    pub fn require_js(
        &self,
        #[wasm_bindgen(
            unchecked_param_type = "Shader | Pass | Mesh | Scene | Array<Shader | Pass | Mesh | Scene>"
        )]
        dependencies: &JsValue,
    ) -> Result<(), JsError> {
        let dependencies = RenderableList::read(dependencies, "Pass.require")?;
        Ok(self.require(&dependencies)?)
    }

    #[wasm_bindgen(js_name = "name")]
    #[lsp_doc("docs/api/core/pass/name.md")]
    pub fn name_js(&self) -> String {
        self.name()
    }

    #[wasm_bindgen(js_name = "loadPrevious")]
    #[lsp_doc("docs/api/core/pass/load_previous.md")]
    pub fn load_previous_js(&self) {
        self.load_previous();
    }

    #[wasm_bindgen(js_name = "addShader")]
    #[lsp_doc("docs/api/core/pass/add_shader.md")]
    pub fn add_shader_js(&self, shader: &Shader) -> Result<(), JsError> {
        Ok(self.add_shader(shader)?)
    }

    #[wasm_bindgen(js_name = "addMesh")]
    #[lsp_doc("docs/api/core/pass/add_mesh.md")]
    pub fn add_mesh_js(&self, mesh: &crate::mesh::Mesh) -> Result<(), JsError> {
        Ok(self.add_mesh(mesh)?)
    }

    /// Unified `Pass.add`, which branches on the runtime JS type. Adding a new
    /// `SceneObject` on the Rust side means adding one try-cast arm here.
    #[wasm_bindgen(js_name = "add")]
    #[lsp_doc("docs/api/core/pass/add.md")]
    pub fn add_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "Model | Camera | Light")] object: &JsValue,
    ) -> Result<Pass, JsError> {
        if let Ok(model) = crate::Model::try_from(object) {
            self.add(&model)?;
        } else if let Ok(camera) = crate::scene::Camera::try_from(object) {
            self.add(&camera)?;
        } else if let Ok(light) = crate::scene::Light::try_from(object) {
            self.add(&light)?;
        } else {
            return Err(JsError::new("Pass.add: expected a Model, Camera, or Light"));
        }
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = "setClearColor")]
    #[lsp_doc("docs/api/core/pass/set_clear_color.md")]
    pub fn set_clear_color_js(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number[] | Float32Array")] color: &JsValue,
    ) -> Result<(), JsError> {
        Ok(self.set_clear_color(Color::try_from(color)?)?)
    }

    #[wasm_bindgen(js_name = "setComputeDispatch")]
    #[lsp_doc("docs/api/core/pass/set_compute_dispatch.md")]
    pub fn set_compute_dispatch_js(&self, x: f64, y: f64, z: f64) -> Result<(), JsError> {
        let x = read_unsigned(x, "Pass.setComputeDispatch: x")?;
        let y = read_unsigned(y, "Pass.setComputeDispatch: y")?;
        let z = read_unsigned(z, "Pass.setComputeDispatch: z")?;
        self.object.set_compute_dispatch(x, y, z);
        Ok(())
    }

    #[wasm_bindgen(js_name = "setViewport")]
    #[lsp_doc("docs/api/core/pass/set_viewport.md")]
    pub fn set_viewport_js(&self, region: &JsValue) -> Result<(), JsError> {
        let region: crate::ScreenRegion = region.try_into()?;
        Ok(self.set_viewport(region)?)
    }

    #[wasm_bindgen(js_name = "setTarget")]
    #[lsp_doc("docs/api/core/pass/set_target.md")]
    pub fn set_target_js(&self, target: &JsValue) -> Result<(), JsError> {
        if let Ok(tt) = TextureTarget::try_from(target) {
            return Ok(self.set_target(&tt)?);
        }

        if let Ok(tex) = Texture::try_from(target) {
            return self
                .set_target(&tex)
                .map_err(|e| JsError::new(&format!("{}", e)));
        }
        Err(JsError::new("setTarget: expected TextureTarget or Texture"))
    }

    #[wasm_bindgen(js_name = "setDepthTarget")]
    #[lsp_doc("docs/api/core/pass/set_depth_target.md")]
    pub fn set_depth_target_js(&self, target: &JsValue) -> Result<(), JsError> {
        if let Ok(tt) = TextureTarget::try_from(target) {
            return Ok(self.set_depth_target(&tt)?);
        }
        if let Ok(tex) = Texture::try_from(target) {
            return self
                .set_depth_target(&tex)
                .map_err(|e| JsError::new(&format!("{}", e)));
        }
        Err(JsError::new(
            "setDepthTarget: expected TextureTarget or Texture",
        ))
    }

    #[wasm_bindgen(js_name = "isCompute")]
    #[lsp_doc("docs/api/core/pass/is_compute.md")]
    pub fn is_compute_js(&self) -> bool {
        self.is_compute()
    }
}
