#![cfg(mobile)]

use lsp_doc::lsp_doc;
use std::sync::Arc;

use crate::scene::Model;
use crate::renderer::platform::mobile::FragmentColorError;
use crate::{Material, Mesh};

#[uniffi::export]
impl Model {
    #[uniffi::constructor(name = "new")]
    #[lsp_doc("docs/api/scene/model/new.md")]
    pub fn new_mobile(mesh: Arc<Mesh>, material: Arc<Material>) -> Arc<Self> {
        Arc::new(Model::new((*mesh).clone(), material_share(&material)))
    }

    #[uniffi::method(name = "mesh")]
    #[lsp_doc("docs/api/scene/model/mesh.md")]
    pub fn mesh_mobile(self: Arc<Self>) -> Arc<Mesh> {
        Arc::new(self.mesh.clone())
    }

    #[uniffi::method(name = "material")]
    #[lsp_doc("docs/api/scene/model/material.md")]
    pub fn material_mobile(self: Arc<Self>) -> Arc<Material> {
        Arc::new(material_share(&self.material))
    }

    #[uniffi::method(name = "transform")]
    #[lsp_doc("docs/api/scene/model/transform.md")]
    pub fn transform_mobile(self: Arc<Self>) -> Vec<f32> {
        let cols = self.transform();
        let mut out = Vec::with_capacity(16);
        for col in cols.iter() {
            out.extend_from_slice(col);
        }
        out
    }

    #[uniffi::method(name = "setTransform")]
    #[lsp_doc("docs/api/scene/model/set_transform.md")]
    pub fn set_transform_mobile(
        self: Arc<Self>,
        matrix: Vec<f32>,
    ) -> Result<(), FragmentColorError> {
        if matrix.len() != 16 {
            return Err(FragmentColorError::Render(
                "Model.setTransform: expected 16 floats (column-major mat4)".into(),
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

    #[uniffi::method(name = "translate")]
    #[lsp_doc("docs/api/scene/model/translate.md")]
    pub fn translate_mobile(
        self: Arc<Self>,
        offset: Vec<f32>,
    ) -> Result<(), FragmentColorError> {
        let arr = take_vec3(&offset, "translate")?;
        self.translate(arr);
        Ok(())
    }

    #[uniffi::method(name = "rotate")]
    #[lsp_doc("docs/api/scene/model/rotate.md")]
    pub fn rotate_mobile(
        self: Arc<Self>,
        axis: Vec<f32>,
        radians: f32,
    ) -> Result<(), FragmentColorError> {
        let arr = take_vec3(&axis, "rotate")?;
        self.rotate(arr, radians);
        Ok(())
    }

    #[uniffi::method(name = "scale")]
    #[lsp_doc("docs/api/scene/model/scale.md")]
    pub fn scale_mobile(
        self: Arc<Self>,
        factor: Vec<f32>,
    ) -> Result<(), FragmentColorError> {
        let arr = take_vec3(&factor, "scale")?;
        self.scale(arr);
        Ok(())
    }
}

fn material_share(material: &Material) -> Material {
    Material::custom(material.shader.clone())
}

fn take_vec3(v: &[f32], field: &str) -> Result<[f32; 3], FragmentColorError> {
    if v.len() != 3 {
        return Err(FragmentColorError::Render(format!(
            "Model.{field}: expected an array of length 3, got {}",
            v.len()
        )));
    }
    Ok([v[0], v[1], v[2]])
}
