use crate::blend::ComplexBlend;
use enum_map::{enum_map, EnumMap};
use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor, ShaderDefValue,
};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Shaders {
    pub color_shader: wgpu::ShaderModule,
    pub bitmap_shader: wgpu::ShaderModule,
    pub gradient_shader: wgpu::ShaderModule,
    pub copy_shader: wgpu::ShaderModule,
}

impl Shaders {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut composer = composer().expect("Couldn't create shader composer");
        let mut shader_defs = HashMap::new();

        let color_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "color.wgsl",
            include_str!("../shaders/color.wgsl"),
        );
        let bitmap_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "bitmap.wgsl",
            include_str!("../shaders/bitmap.wgsl"),
        );
        let gradient_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "gradient.wgsl",
            include_str!("../shaders/gradient.wgsl"),
        );
        let copy_shader = make_shader(
            device,
            &mut composer,
            &shader_defs,
            "copy.wgsl",
            include_str!("../shaders/copy.wgsl"),
        );

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
        source: include_str!("../shaders/common.wgsl"),
        file_path: "common.wgsl",
        ..Default::default()
    })?;
    composer.add_composable_module(ComposableModuleDescriptor {
        source: ruffle_render::shader_source::SHADER_FILTER_COMMON,
        file_path: "shader_filter_common.wgsl",
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
) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: create_debug_label!("Shader {}", name).as_deref(),
        source: wgpu::ShaderSource::Naga(Cow::Owned(
            composer
                .make_naga_module(NagaModuleDescriptor {
                    source,
                    file_path: name,
                    shader_defs: shader_defs.clone(),
                    ..Default::default()
                })
                .unwrap_or_else(|e| {
                    panic!(
                        "{name} failed to compile:\n{}\n{:#?}",
                        e.emit_to_string(composer),
                        e
                    )
                }),
        )),
    })
}
