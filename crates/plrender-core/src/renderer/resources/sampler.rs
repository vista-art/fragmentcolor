pub(super) trait SamplerSelector {
    fn get_blueprint(options: BlueprintOptions) -> SamplerBlueprint;
    fn from_blueprint(device: &wgpu::Device, bp: SamplerBlueprint) -> wgpu::Sampler;
}

pub struct BlueprintOptions {
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub smooth: bool,
}

impl Default for BlueprintOptions {
    fn default() -> Self {
        Self {
            repeat_x: true,
            repeat_y: true,
            smooth: true,
        }
    }
}

pub(super) struct SamplerBlueprint {
    pub(super) address_mode_u: wgpu::AddressMode,
    pub(super) address_mode_v: wgpu::AddressMode,
    pub(super) filter: wgpu::FilterMode,
    pub(super) label: String,
    pub(super) compare: Option<wgpu::CompareFunction>,
}

impl SamplerSelector for wgpu::Sampler {
    fn get_blueprint(options: BlueprintOptions) -> SamplerBlueprint {
        match (options.repeat_x, options.repeat_y, options.smooth) {
            (true, true, true) => SamplerBlueprint {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                filter: wgpu::FilterMode::Linear,
                label: String::from("Repeat & Linear sampler"),
                compare: None,
            },
            (true, true, false) => SamplerBlueprint {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                filter: wgpu::FilterMode::Nearest,
                label: String::from("Repeat & Nearest sampler"),
                compare: None,
            },
            (false, false, true) => SamplerBlueprint {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                filter: wgpu::FilterMode::Linear,
                label: String::from("Clamp & Linear sampler"),
                compare: None,
            },
            (false, false, false) => SamplerBlueprint {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                filter: wgpu::FilterMode::Nearest,
                label: String::from("Clamp & Nearest sampler"),
                compare: None,
            },
            (false, true, true) => SamplerBlueprint {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::Repeat,
                filter: wgpu::FilterMode::Linear,
                label: String::from("Clamp U, Repeat V & Linear sampler"),
                compare: None,
            },
            (false, true, false) => SamplerBlueprint {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::Repeat,
                filter: wgpu::FilterMode::Nearest,
                label: String::from("Clamp U, Repeat V & Nearest sampler"),
                compare: None,
            },
            (true, false, true) => SamplerBlueprint {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                filter: wgpu::FilterMode::Linear,
                label: String::from("Repeat U, Clamp V & Linear sampler"),
                compare: None,
            },
            (true, false, false) => SamplerBlueprint {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                filter: wgpu::FilterMode::Nearest,
                label: String::from("Repeat U, Clamp V & Nearest sampler"),
                compare: None,
            },
        }
    }

    fn from_blueprint(device: &wgpu::Device, bp: SamplerBlueprint) -> wgpu::Sampler {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(bp.label.as_str()),
            address_mode_u: bp.address_mode_u,
            address_mode_v: bp.address_mode_v,
            address_mode_w: bp.address_mode_v,
            mag_filter: bp.filter,
            min_filter: bp.filter,
            mipmap_filter: bp.filter,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: bp.compare,
            anisotropy_clamp: 1,
            border_color: None,
        });
        sampler
    }
}
