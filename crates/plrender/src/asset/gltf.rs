use crate::{scene::components::Light, MeshBuilder};
use std::{collections::VecDeque, ops, path::Path};

#[derive(Default)]
struct MeshScratch {
    indices: Vec<u16>,
    positions: Vec<crate::geometry::vertex::Position>,
    tex_coords: Vec<crate::geometry::vertex::TextureCoordinates>,
    normals: Vec<crate::geometry::vertex::Normal>,
}

struct Texture {
    image: crate::TextureId,
}

struct Primitive {
    mesh: crate::Bundle,
    color: crate::Color,
    shader: crate::renderer::renderpass::Shader,
    material: crate::renderer::renderpass::Material,
}

fn load_texture(mut data: gltf::image::Data, renderer: &mut crate::Renderer) -> Texture {
    let format = match data.format {
        gltf::image::Format::R8 => wgpu::TextureFormat::R8Unorm,
        gltf::image::Format::R8G8 => wgpu::TextureFormat::Rg8Unorm,
        gltf::image::Format::R8G8B8 | gltf::image::Format::B8G8R8 => {
            log::warn!(
                "Converting {}x{} texture from RGB to RGBA...",
                data.width,
                data.height
            );
            let original = data.pixels;
            data.pixels = Vec::with_capacity(original.len() * 4 / 3);
            for chunk in original.chunks(3) {
                data.pixels.push(chunk[0]);
                data.pixels.push(chunk[1]);
                data.pixels.push(chunk[2]);
                data.pixels.push(0xFF);
            }
            if data.format == gltf::image::Format::R8G8B8 {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Bgra8UnormSrgb
            }
        }
        gltf::image::Format::R16G16B16 => panic!("RGB16 is outdated"),
        gltf::image::Format::R8G8B8A8 => wgpu::TextureFormat::Rgba8UnormSrgb,
        gltf::image::Format::B8G8R8A8 => wgpu::TextureFormat::Bgra8UnormSrgb,
        gltf::image::Format::R16 => wgpu::TextureFormat::R16Float,
        gltf::image::Format::R16G16 => wgpu::TextureFormat::Rg16Float,
        gltf::image::Format::R16G16B16A16 => wgpu::TextureFormat::Rgba16Float,
    };

    let desc = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[format],
    };

    let image = renderer.add_texture_from_bytes(&desc, &data.pixels);
    Texture { image }
}

fn load_primitive<'a>(
    primitive: gltf::Primitive<'a>,
    buffers: &[gltf::buffer::Data],
    textures: &[Texture],
    renderer: &mut crate::Renderer,
    scratch: &mut MeshScratch,
) -> Primitive {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));

    let mesh_builder = MeshBuilder::new(renderer);

    if let Some(indices) = reader.read_indices() {
        scratch.indices.clear();
        scratch.indices.extend(indices.into_u32().map(|i| i as u16));
        mesh_builder.index(&scratch.indices);
    }

    if let Some(positions) = reader.read_positions() {
        scratch.positions.clear();
        scratch.positions.extend(positions.map(crate::Position));
        mesh_builder.vertex(&scratch.positions);
    }

    if let Some(tex_coords) = reader.read_tex_coords(0) {
        scratch.tex_coords.clear();
        scratch
            .tex_coords
            .extend(tex_coords.into_u16().map(crate::TextureCoordinates));
        mesh_builder.vertex(&scratch.tex_coords);
    }

    if let Some(normals) = reader.read_normals() {
        scratch.normals.clear();
        scratch.normals.extend(normals.map(crate::Normal));
        mesh_builder.vertex(&scratch.normals);
    }

    let mat = primitive.material();
    let pbr = mat.pbr_metallic_roughness();
    let base_color = pbr.base_color_factor();
    let material = crate::pass::Material {
        base_color_map: pbr
            .base_color_texture()
            .map(|t| textures[t.texture().index()].image),
        emissive_color: crate::Color::from_rgb_alpha(mat.emissive_factor(), 0.0),
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
        normal_scale: 1.0,
        occlusion_strength: 1.0,
    };

    Primitive {
        mesh: mesh_builder.build(),
        color: crate::Color::from_rgba(base_color),
        shader: crate::pass::Shader::Gouraud { flat: true },
        material,
    }
}

#[derive(Debug)]
struct Named<T> {
    data: T,
    name: Option<String>,
}

#[derive(Debug)]
pub struct NamedVec<T>(Vec<Named<T>>);

impl<T> Default for NamedVec<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> ops::Index<usize> for NamedVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.0[index].data
    }
}

impl<T> NamedVec<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().map(|elem| &elem.data)
    }

    pub fn find(&self, name: &str) -> Option<&T> {
        self.0
            .iter()
            .find(|elem| elem.name.as_deref() == Some(name))
            .map(|elem| &elem.data)
    }
}

#[derive(Default)]
pub struct Module {
    pub entities: NamedVec<plr::EntityId>,
    pub cameras: NamedVec<plr::Camera>,
}

/// Load mesh from glTF 2.0 format.
pub fn load_gltf(
    path: impl AsRef<Path>,
    scene: &mut crate::Scene,
    global_parent: crate::NodeId,
    renderer: &mut crate::Renderer,
) -> Module {
    let mut module = Module::default();
    let (gltf, buffers, images) = gltf::import(path).expect("invalid glTF 2.0");

    let mut textures = Vec::with_capacity(images.len());
    for (_texture, data) in gltf.textures().zip(images.into_iter()) {
        let texture = load_texture(data, renderer);
        textures.push(texture);
    }

    let mut prototypes = Vec::with_capacity(gltf.meshes().len());
    let mut scratch = MeshScratch::default();
    for gltf_mesh in gltf.meshes() {
        let mut primitives = Vec::new();
        for gltf_primitive in gltf_mesh.primitives() {
            let primitive =
                load_primitive(gltf_primitive, &buffers, &textures, renderer, &mut scratch);
            primitives.push(primitive);
        }
        prototypes.push(primitives);
    }

    struct PreNode<'a> {
        gltf_node: gltf::Node<'a>,
        parent: crate::NodeId,
    }

    let mut deque = VecDeque::new();
    for gltf_scene in gltf.scenes() {
        deque.extend(gltf_scene.nodes().map(|gltf_node| PreNode {
            gltf_node,
            parent: global_parent,
        }));
    }

    while let Some(PreNode { gltf_node, parent }) = deque.pop_front() {
        log::debug!("Node {:?}", gltf_node.name());

        let (translation, rotation, scale) = gltf_node.transform().decomposed();
        let uniform_scale = if scale[1] != scale[0] || scale[2] != scale[0] {
            log::warn!(
                "Node[{}] scale {:?} is non-uniform",
                gltf_node.index(),
                scale
            );
            (scale[0] + scale[1] + scale[2]) / 3.0
        } else {
            scale[0]
        };

        let node = scene
            .add_node()
            .parent(parent)
            .position(translation.into())
            .orientation(rotation.into())
            .scale(uniform_scale)
            .build();

        for gltf_child in gltf_node.children() {
            deque.push_back(PreNode {
                gltf_node: gltf_child,
                parent: node,
            });
        }

        if let Some(gltf_mesh) = gltf_node.mesh() {
            log::debug!("Mesh {:?}", gltf_mesh.name());
            for primitive in prototypes[gltf_mesh.index()].iter() {
                let entity = scene
                    .add_entity(&primitive.mesh)
                    .component(primitive.color)
                    .component(primitive.shader)
                    .component(primitive.material)
                    .parent(node)
                    .build();
                module.entities.0.push(Named {
                    data: entity,
                    name: gltf_mesh.name().map(str::to_string),
                });
            }
        }

        if let Some(gltf_camera) = gltf_node.camera() {
            let (depth, projection) = match gltf_camera.projection() {
                gltf::camera::Projection::Orthographic(p) => (
                    p.znear()..p.zfar(),
                    plr::Projection::Orthographic {
                        center: [0.0; 2].into(),
                        //Note: p.xmag() is ignored
                        extent_y: p.ymag(),
                    },
                ),
                gltf::camera::Projection::Perspective(p) => (
                    p.znear()..p.zfar().unwrap_or(f32::INFINITY),
                    plr::Projection::Perspective {
                        fov_y: p.yfov().to_degrees(),
                    },
                ),
            };
            log::debug!(
                "Camera {:?} depth {:?} proj {:?} at {:?}",
                gltf_camera.name(),
                depth,
                projection,
                scene[node]
            );
            module.cameras.0.push(Named {
                data: plr::Camera {
                    projection,
                    depth,
                    node,
                    background: plr::Color::default(),
                },
                name: gltf_camera.name().map(str::to_string),
            });
        }

        if let Some(gltf_light) = gltf_node.light() {
            use gltf::khr_lights_punctual::Kind as LightType;
            let light_type = match gltf_light.kind() {
                LightType::Directional => plr::LightType::Directional,
                LightType::Point => plr::LightType::Point,
                LightType::Spot { .. } => {
                    log::warn!("Spot lights are not supported: {:?}", gltf_light.name());
                    continue;
                }
            };

            let light = Light {
                node,
                color: crate::Color::from_rgb_alpha(gltf_light.color(), 0.0),
                intensity: gltf_light.intensity(),
                variant: light_type,
            };
            let mut builder = hecs::EntityBuilder::new();
            let light_entity = builder.add(light).build();
            self.scene.add(light_entity);

            module.entities.0.push(Named {
                data: light,
                name: gltf_light.name().map(str::to_string),
            });
        }
    }

    module
}
