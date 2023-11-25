use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor, ShaderDefValue,
};
use std::borrow::Cow;
use std::collections::HashMap;

// @TODO integrate the composer in an Object

#[derive(Debug)]
pub struct Shaders {
    pub color_shader: Option<wgpu::ShaderModule>,
    pub bitmap_shader: Option<wgpu::ShaderModule>,
    pub gradient_shader: Option<wgpu::ShaderModule>,
    pub copy_shader: Option<wgpu::ShaderModule>,
}

#[allow(dead_code)]
impl Shaders {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut composer = composer().expect("Couldn't create shader composer");
        let shader_defs = HashMap::new();

        let color_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "color.wgsl",
            include_str!("./shaders/references/ruffle/color.wgsl"),
        )
        .ok();
        let bitmap_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "bitmap.wgsl",
            include_str!("./shaders/references/ruffle/bitmap.wgsl"),
        )
        .ok();
        let gradient_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "gradient.wgsl",
            include_str!("./shaders/references/ruffle/gradient.wgsl"),
        )
        .ok();
        let copy_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "copy.wgsl",
            include_str!("./shaders/references/ruffle/copy.wgsl"),
        )
        .ok();

        Self {
            color_shader,
            bitmap_shader,
            gradient_shader,
            copy_shader,
        }
    }
}

fn composer() -> Result<Composer, ComposerError> {
    let mut composer = Composer::default();
    // [NA] Hack to get all capabilities since nobody exposes this type easily
    let capabilities = composer.capabilities;
    composer = composer.with_capabilities(!capabilities);
    composer.add_composable_module(ComposableModuleDescriptor {
        source: include_str!("./shaders/common.wgsl"),
        file_path: "common.wgsl",
        ..Default::default()
    })?;
    composer.add_composable_module(ComposableModuleDescriptor {
        source: include_str!("./shaders/references/ruffle/shader_filter_common.wgsl"),
        file_path: "./shaders/references/ruffle/shader_filter_common.wgsl",
        ..Default::default()
    })?;
    Ok(composer)
}

fn make_shader(
    device: &wgpu::Device,
    composer: &mut Composer,
    shader_defs: &HashMap<String, ShaderDefValue>,
    name: &str,
    source: &'static str,
) -> Result<wgpu::ShaderModule, ComposerError> {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(format!("Shader {}", name).as_str()),
        source: wgpu::ShaderSource::Naga(Cow::Owned(composer.make_naga_module(
            NagaModuleDescriptor {
                source,
                file_path: name,
                shader_defs: shader_defs.clone(),
                ..Default::default()
            },
        )?)),
    });

    Ok(shader_module)
}
