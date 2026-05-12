#![cfg(wasm)]

use lsp_doc::lsp_doc;
use wasm_bindgen::prelude::*;

use crate::scene::Model;
use crate::{Material, Mesh};

#[wasm_bindgen]
impl Model {
    #[wasm_bindgen(constructor)]
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new_js(mesh: &Mesh, material: &Material) -> Self {
        // Materials and Meshes are wrapped around Arc internally, so cloning
        // here is an Arc::clone — cheap, share semantics. The JS caller keeps
        // their original handle live.
        Model::new(mesh.clone(), material_share(material))
    }

    #[wasm_bindgen(js_name = "mesh")]
    #[lsp_doc("docs/api/scene/model/mesh.md")]
    pub fn mesh_js(&self) -> Mesh {
        self.mesh.clone()
    }

    #[wasm_bindgen(js_name = "material")]
    #[lsp_doc("docs/api/scene/model/material.md")]
    pub fn material_js(&self) -> Material {
        material_share(&self.material)
    }

    #[wasm_bindgen(js_name = "transform")]
    #[lsp_doc("docs/api/scene/model/transform.md")]
    pub fn transform_js(&self) -> Vec<f32> {
        let cols = self.transform();
        let mut flat = Vec::with_capacity(16);
        for col in cols.iter() {
            flat.extend_from_slice(col);
        }
        flat
    }

    #[wasm_bindgen(js_name = "setTransform")]
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform_js(&self, matrix: Vec<f32>) -> Result<(), JsError> {
        if matrix.len() != 16 {
            return Err(JsError::new(
                "Model.setTransform: expected 16 floats (column-major mat4)",
            ));
        }
        let m = [
            [matrix[0], matrix[1], matrix[2], matrix[3]],
            [matrix[4], matrix[5], matrix[6], matrix[7]],
            [matrix[8], matrix[9], matrix[10], matrix[11]],
            [matrix[12], matrix[13], matrix[14], matrix[15]],
        ];
        self.set_transform(m);
        Ok(())
    }

    #[wasm_bindgen(js_name = "translate")]
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate_js(&self, offset: Vec<f32>) -> Result<(), JsError> {
        let arr = vec3(&offset, "translate")?;
        self.translate(arr);
        Ok(())
    }

    #[wasm_bindgen(js_name = "rotate")]
    #[lsp_doc("docs/api/scene/model/rotate.md")]
    pub fn rotate_js(&self, axis: Vec<f32>, radians: f32) -> Result<(), JsError> {
        let arr = vec3(&axis, "rotate")?;
        self.rotate(arr, radians);
        Ok(())
    }

    #[wasm_bindgen(js_name = "scale")]
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale_js(&self, factor: Vec<f32>) -> Result<(), JsError> {
        let arr = vec3(&factor, "scale")?;
        self.scale(arr);
        Ok(())
    }
}

fn material_share(material: &Material) -> Material {
    // Share the Material's shader (Arc-clone) so the JS handle and the new
    // Material observe the same uniform state. Use Material::clone() when you
    // want an independent copy (it routes through Shader::duplicate).
    Material::custom(material.shader.clone())
}

fn vec3(v: &[f32], field: &str) -> Result<[f32; 3], JsError> {
    if v.len() != 3 {
        return Err(JsError::new(&format!(
            "Model.{field}: expected an array of length 3"
        )));
    }
    Ok([v[0], v[1], v[2]])
}
