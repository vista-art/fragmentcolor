use image::GenericImageView;
use std::error::Error;

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            // This is the same as with the SurfaceConfig. It
            // specifies what texture formats can be used to
            // create TextureViews for this texture. The base
            // texture format (Rgba8UnormSrgb in this case) is
            // always supported. Note that using a different
            // texture format is not supported on the WebGL2
            // backend.
            view_formats: &[],
        });

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            // The actual pixel data
            &rgba,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        // We don't need to configure the texture view much, so let's let wgpu define it.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    // We need the DEPTH_FORMAT for when we create the depth stage of
    // the render_pipeline and for creating the depth texture itself.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            // Our depth texture needs to be the same size as our screen if we
            // want things to render correctly. We can use our config to make sure
            // that our depth texture is the same size as our surface textures.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            // Since we are rendering to this texture,
            // we need to add the RENDER_ATTACHMENT flag to it.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = create_sampler(
            device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Linear,
            "Clamp & Linear sampler",
            Some(wgpu::CompareFunction::LessEqual),
        );

        Self {
            texture,
            view,
            sampler,
        }
    }
}

#[derive(Debug)]
pub struct Samplers {
    pub repeat_linear: wgpu::Sampler,
    pub repeat_nearest: wgpu::Sampler,
    pub clamp_linear: wgpu::Sampler,
    pub clamp_nearest: wgpu::Sampler,
    pub clamp_u_repeat_v_linear: wgpu::Sampler,
    pub clamp_u_repeat_v_nearest: wgpu::Sampler,
    pub repeat_u_clamp_v_linear: wgpu::Sampler,
    pub repeat_u_clamp_v_nearest: wgpu::Sampler,
}

pub struct SamplerOptions {
    pub repeat_x: bool,
    pub repeat_y: bool,
    pub smooth: bool,
}

impl Samplers {
    pub fn new(device: &wgpu::Device) -> Self {
        let repeat_linear = create_sampler(
            device,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
            wgpu::FilterMode::Linear,
            "Repeat & Linear sampler",
            None,
        );
        let repeat_nearest = create_sampler(
            device,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::Repeat,
            wgpu::FilterMode::Nearest,
            "Repeat & Nearest sampler",
            None,
        );
        let clamp_linear = create_sampler(
            device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Linear,
            "Clamp & Linear sampler",
            None,
        );
        let clamp_nearest = create_sampler(
            device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Nearest,
            "Clamp & Nearest sampler",
            None,
        );

        let clamp_u_repeat_v_linear = create_sampler(
            device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::AddressMode::Repeat,
            wgpu::FilterMode::Linear,
            "Clamp U, Repeat V & Linear sampler",
            None,
        );

        let clamp_u_repeat_v_nearest = create_sampler(
            device,
            wgpu::AddressMode::ClampToEdge,
            wgpu::AddressMode::Repeat,
            wgpu::FilterMode::Nearest,
            "Clamp U, Repeat V & Nearest sampler",
            None,
        );

        let repeat_u_clamp_v_linear = create_sampler(
            device,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Linear,
            "Repeat U, Clamp V & Linear sampler",
            None,
        );

        let repeat_u_clamp_v_nearest = create_sampler(
            device,
            wgpu::AddressMode::Repeat,
            wgpu::AddressMode::ClampToEdge,
            wgpu::FilterMode::Nearest,
            "Repeat U, Clamp V & Nearest sampler",
            None,
        );

        Self {
            repeat_linear,
            repeat_nearest,
            clamp_linear,
            clamp_nearest,
            clamp_u_repeat_v_linear,
            clamp_u_repeat_v_nearest,
            repeat_u_clamp_v_linear,
            repeat_u_clamp_v_nearest,
        }
    }

    pub fn get_sampler(&self, options: SamplerOptions) -> &wgpu::Sampler {
        match (options.repeat_x, options.repeat_y, options.smooth) {
            (true, true, true) => &self.repeat_linear,
            (true, true, false) => &self.repeat_nearest,
            (false, false, true) => &self.clamp_linear,
            (false, false, false) => &self.clamp_nearest,
            (false, true, true) => &self.clamp_u_repeat_v_linear,
            (false, true, false) => &self.clamp_u_repeat_v_nearest,
            (true, false, true) => &self.repeat_u_clamp_v_linear,
            (true, false, false) => &self.repeat_u_clamp_v_nearest,
        }
    }
}

fn create_sampler(
    device: &wgpu::Device,
    address_mode_u: wgpu::AddressMode,
    address_mode_v: wgpu::AddressMode,
    filter: wgpu::FilterMode,
    sampler_label: &str,
    compare: Option<wgpu::CompareFunction>,
) -> wgpu::Sampler {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(sampler_label),
        address_mode_u,
        address_mode_v,
        address_mode_w: address_mode_v,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        compare: None,
        anisotropy_clamp: 1,
        border_color: None,
    });
    sampler
}
