#![allow(dead_code)]

use crate::{
    components::{self, Color, Mesh},
    math::geometry::vertex,
    renderer::Renderer,
    resources::{
        self,
        mesh::{BuiltMesh, MeshBuilder},
        texture::TextureId,
    },
    scene, SceneObject, Vec2,
};
use std::{collections::VecDeque, ops, path::Path};

#[derive(Default)]
struct MeshScratch {
    indices: Vec<u16>,
    positions: Vec<vertex::Position>,
    tex_coords: Vec<vertex::TextureCoordinates>,
    normals: Vec<vertex::Normal>,
}

struct Texture {
    image: TextureId,
}

struct Primitive {
    mesh: Option<BuiltMesh>,
    color: Color,
    shader: renderpass::ShaderType,
    material: renderpass::Material,
}

fn load_texture(renderer: &Renderer, data: gltf::image::Data) -> Texture {
    let (texture_id, _) = resources::Texture::from_bytes(renderer, &data.pixels).unwrap();
    Texture { image: texture_id }
}

fn load_primitive(
    renderer: &Renderer,
    primitive: gltf::Primitive,
    buffers: &[gltf::buffer::Data],
    textures: &[Texture],
    scratch: &mut MeshScratch,
) -> Primitive {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));

    let mut mesh_builder = MeshBuilder::new();

    if let Some(indices) = reader.read_indices() {
        scratch.indices.clear();
        scratch.indices.extend(indices.into_u32().map(|i| i as u16));
        mesh_builder.index(&scratch.indices);
    }

    if let Some(positions) = reader.read_positions() {
        scratch.positions.clear();
        scratch.positions.extend(positions.map(vertex::Position));
        mesh_builder.vertex(&scratch.positions);
    }

    if let Some(tex_coords) = reader.read_tex_coords(0) {
        scratch.tex_coords.clear();
        scratch
            .tex_coords
            .extend(tex_coords.into_u16().map(vertex::TextureCoordinates));
        mesh_builder.vertex(&scratch.tex_coords);
    }

    if let Some(normals) = reader.read_normals() {
        scratch.normals.clear();
        scratch.normals.extend(normals.map(vertex::Normal));
        mesh_builder.vertex(&scratch.normals);
    }

    let mat = primitive.material();
    let pbr = mat.pbr_metallic_roughness();
    let base_color = pbr.base_color_factor();
    let material = renderpass::Material {
        base_color_map: pbr
            .base_color_texture()
            .map(|t| textures[t.texture().index()].image),
        emissive_color: Color::from_rgb_alpha(mat.emissive_factor(), 0.0),
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
        normal_scale: 1.0,
        occlusion_strength: 1.0,
    };

    Primitive {
        mesh: mesh_builder.build(renderer).ok(),
        color: Color::from_rgba(base_color),
        shader: renderpass::ShaderType::Gouraud { flat: true },
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
    pub entities: NamedVec<crate::ObjectId>,
    pub cameras: NamedVec<crate::Camera>,
}

/// Load mesh from glTF 2.0 format.
pub fn load_gltf(
    renderer: &Renderer,
    path: impl AsRef<Path>,
    scene: &mut scene::Scene,
    global_parent: scene::transform::TransformId,
) -> Module {
    let mut module = Module::default();
    let (gltf, buffers, images) = gltf::import(path).expect("invalid glTF 2.0");

    let mut textures = Vec::with_capacity(images.len());
    for (_texture, data) in gltf.textures().zip(images.into_iter()) {
        let texture = load_texture(renderer, data);
        textures.push(texture);
    }

    let mut prototypes = Vec::with_capacity(gltf.meshes().len());
    let mut scratch = MeshScratch::default();
    for gltf_mesh in gltf.meshes() {
        let mut primitives = Vec::new();
        for gltf_primitive in gltf_mesh.primitives() {
            let primitive =
                load_primitive(renderer, gltf_primitive, &buffers, &textures, &mut scratch);
            primitives.push(primitive);
        }
        prototypes.push(primitives);
    }

    struct PreTransform<'a> {
        gltf_node: gltf::Node<'a>,
        parent: scene::transform::TransformId,
    }

    let mut deque = VecDeque::new();
    for gltf_scene in gltf.scenes() {
        deque.extend(gltf_scene.nodes().map(|gltf_node| PreTransform {
            gltf_node,
            parent: global_parent,
        }));
    }

    while let Some(PreTransform { gltf_node, parent }) = deque.pop_front() {
        log::debug!("Transform {:?}", gltf_node.name());

        let (position, rotation, scale) = gltf_node.transform().decomposed();

        let mut empty = components::Empty::new();
        empty
            .set_parent_transform(parent)
            .set_position(position)
            .set_rotation_quaternion(rotation)
            .set_scale(scale);

        scene.add(&mut empty);

        for gltf_child in gltf_node.children() {
            deque.push_back(PreTransform {
                gltf_node: gltf_child,
                parent: empty.transform_id(),
            });
        }

        if let Some(gltf_mesh) = gltf_node.mesh() {
            log::debug!("Mesh {:?}", gltf_mesh.name());
            for primitive in prototypes[gltf_mesh.index()].iter_mut() {
                let mut mesh = Mesh::new(primitive.mesh.clone());
                mesh.add_component(primitive.color)
                    .add_component(primitive.shader)
                    .add_component(primitive.material)
                    .set_parent_transform(empty.transform_id());

                let object_id = scene.add(&mut mesh);

                module.entities.0.push(Named {
                    data: object_id,
                    name: gltf_mesh.name().map(str::to_string),
                });
            }
        }

        if let Some(gltf_camera) = gltf_node.camera() {
            let (depth, projection) = match gltf_camera.projection() {
                gltf::camera::Projection::Orthographic(p) => (
                    p.znear()..p.zfar(),
                    components::Projection::Orthographic {
                        center: [0.0; 2].into(),
                        //Note: p.xmag() is ignored
                        size: Vec2 {
                            x: p.xmag() * 2.0,
                            y: p.ymag() * 2.0,
                        },
                    },
                ),
                gltf::camera::Projection::Perspective(p) => (
                    p.znear()..p.zfar().unwrap_or(f32::INFINITY),
                    components::Projection::Perspective {
                        fov_y: p.yfov().to_degrees(),
                    },
                ),
            };
            log::debug!(
                "Camera {:?} depth {:?} proj {:?} at {:?}",
                gltf_camera.name(),
                depth,
                projection,
                scene[empty.transform_id()]
            );
            module.cameras.0.push(Named {
                data: components::Camera {
                    projection,
                    z_near: depth.start,
                    z_far: depth.end,
                    transform_id: empty.transform_id(),
                },
                name: gltf_camera.name().map(str::to_string),
            });
        }

        if let Some(gltf_light) = gltf_node.light() {
            use gltf::khr_lights_punctual::Kind as LightType;
            let light_type = match gltf_light.kind() {
                LightType::Directional => components::LightType::Directional,
                LightType::Point => components::LightType::Point,
                LightType::Spot { .. } => {
                    log::warn!("Spot lights are not supported: {:?}", gltf_light.name());
                    continue;
                }
            };

            let mut light = components::Light::new(components::LightOptions {
                color: Color::from_rgb_alpha(gltf_light.color(), 0.0),
                intensity: gltf_light.intensity(),
                variant: light_type,
            });

            light.set_parent(&empty);

            let light_id = scene.add(&mut light);

            module.entities.0.push(Named {
                data: light_id,
                name: gltf_light.name().map(str::to_string),
            });
        }
    }

    module
}
