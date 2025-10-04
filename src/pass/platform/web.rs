#![cfg(wasm)]

use crate::{
    Color, Frame, Mesh, Pass, PassError, PassInput, Renderable, Shader, Texture, TextureTarget,
};
use js_sys::Array;
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
    pub fn require_js(&self, dependencies: &JsValue) -> Result<(), PassError> {
        if let Ok(shader) = Shader::try_from(dependencies) {
            return self.require(&shader);
        } else if let Ok(pass) = Pass::try_from(dependencies) {
            return self.require(&pass);
        } else if let Ok(frame) = Frame::try_from(dependencies) {
            return self.require(&frame);
        } else if let Ok(mesh) = Mesh::try_from(dependencies) {
            return self.require(&mesh);
        } else if Array::is_array(dependencies) {
            let deps: Vec<Box<dyn Renderable>> = Array::from(dependencies)
                .into_iter()
                .filter_map(|v| {
                    if let Ok(shader) = Shader::try_from(&v) {
                        Some(Box::new(shader) as Box<dyn Renderable>)
                    } else if let Ok(pass) = Pass::try_from(&v) {
                        Some(Box::new(pass) as Box<dyn Renderable>)
                    } else if let Ok(frame) = Frame::try_from(&v) {
                        Some(Box::new(frame) as Box<dyn Renderable>)
                    } else if let Ok(mesh) = Mesh::try_from(&v) {
                        Some(Box::new(mesh) as Box<dyn Renderable>)
                    } else {
                        None
                    }
                })
                .collect();

            return self.require(&deps);
        } else {
            return Ok(());
        }
    }

    #[wasm_bindgen(js_name = "loadPrevious")]
    #[lsp_doc("docs/api/core/pass/load_previous.md")]
    pub fn load_previous_js(&self) {
        *self.object.input.write() = PassInput::load();
    }

    #[wasm_bindgen(js_name = "getInput")]
    #[lsp_doc("docs/api/core/pass/get_input.md")]
    pub fn get_input_js(&self) -> PassInput {
        self.object.get_input()
    }

    #[wasm_bindgen(js_name = "addShader")]
    #[lsp_doc("docs/api/core/pass/add_shader.md")]
    pub fn add_shader_js(&self, shader: &Shader) {
        self.object.add_shader(shader);
    }

    #[wasm_bindgen(js_name = "addMesh")]
    #[lsp_doc("docs/api/core/pass/add_mesh.md")]
    pub fn add_mesh_js(&self, mesh: &crate::mesh::Mesh) -> Result<(), JsError> {
        Ok(self.add_mesh(mesh)?)
    }

    #[wasm_bindgen(js_name = "addMeshToShader")]
    #[lsp_doc("docs/api/core/pass/add_mesh_to_shader.md")]
    pub fn add_mesh_to_shader_js(
        &self,
        mesh: &crate::mesh::Mesh,
        shader: &Shader,
    ) -> Result<(), JsError> {
        Ok(self.add_mesh_to_shader(mesh, shader)?)
    }

    #[wasm_bindgen(js_name = "setClearColor")]
    #[lsp_doc("docs/api/core/pass/set_clear_color.md")]
    pub fn set_clear_color_js(&self, color: &JsValue) -> Result<(), JsError> {
        let color: Color = color.try_into()?;
        self.object.set_clear_color(color);
        Ok(())
    }

    #[wasm_bindgen(js_name = "setComputeDispatch")]
    #[lsp_doc("docs/api/core/pass/set_compute_dispatch.md")]
    pub fn set_compute_dispatch_js(&self, x: u32, y: u32, z: u32) {
        self.object.set_compute_dispatch(x, y, z);
    }

    #[wasm_bindgen(js_name = "setViewport")]
    #[lsp_doc("docs/api/core/pass/set_viewport.md")]
    pub fn set_viewport_js(&self, region: &JsValue) -> Result<(), JsError> {
        let r: crate::Region = region.try_into()?;
        self.set_viewport(r);
        Ok(())
    }

    #[wasm_bindgen(js_name = "addTarget")]
    #[lsp_doc("docs/api/core/pass/add_target.md")]
    pub fn add_target_js(&self, target: &JsValue) -> Result<(), JsError> {
        if let Ok(tt) = TextureTarget::try_from(target) {
            return Ok(self.add_target(&tt)?);
        }

        if let Ok(tex) = Texture::try_from(target) {
            return self
                .add_target(&tex)
                .map_err(|e| JsError::new(&format!("{}", e)));
        }
        Err(JsError::new("addTarget: expected TextureTarget or Texture"))
    }

    #[wasm_bindgen(js_name = "addDepthTarget")]
    #[lsp_doc("docs/api/core/pass/add_depth_target.md")]
    pub fn add_depth_target_js(&self, target: &JsValue) -> Result<(), JsError> {
        if let Ok(tt) = TextureTarget::try_from(target) {
            return Ok(self.add_depth_target(&tt)?);
        }
        if let Ok(tex) = Texture::try_from(target) {
            return self
                .add_depth_target(&tex)
                .map_err(|e| JsError::new(&format!("{}", e)));
        }
        Err(JsError::new(
            "addDepthTarget: expected TextureTarget or Texture",
        ))
    }

    #[wasm_bindgen(js_name = "isCompute")]
    #[lsp_doc("docs/api/core/shader/is_compute.md")]
    pub fn is_compute_js(&self) -> bool {
        self.is_compute()
    }
}
