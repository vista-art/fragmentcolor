#[derive(Clone, Debug, PartialEq)]
struct FunctionParameter {
    pub name: &'static str,
    pub type_name: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
struct FunctionSignature {
    pub name: &'static str,
    pub parameters: &'static [FunctionParameter],
    pub return_type: Option<&'static str>,
}

static API_MAP: phf::Map<&'static str, &[FunctionSignature]> = ::phf::Map {
    key: 8694567506910003252,
    disps: &[
        (4, 23),
        (1, 0),
        (0, 7),
        (0, 9),
        (0, 12),
        (2, 1),
        (0, 1),
        (7, 38),
    ],
    entries: &[
        ("PhongConfig", &[]),
        (
            "RendererBuilder",
            &[
                FunctionSignature {
                    name: "power_hungry",
                    parameters: &[FunctionParameter {
                        name: "hungry",
                        type_name: "bool",
                    }],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "build_offscreen",
                    parameters: &[],
                    return_type: Some("Renderer"),
                },
                FunctionSignature {
                    name: "software",
                    parameters: &[FunctionParameter {
                        name: "software",
                        type_name: "bool",
                    }],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "build",
                    parameters: &[FunctionParameter {
                        name: "window",
                        type_name: "& W",
                    }],
                    return_type: Some("Renderer"),
                },
            ],
        ),
        (
            "renderer_resources_sampler.rs",
            &[
                FunctionSignature {
                    name: "create_default_sampler",
                    parameters: &[FunctionParameter {
                        name: "device",
                        type_name: "& wgpu :: Device",
                    }],
                    return_type: Some("wgpu :: Sampler"),
                },
                FunctionSignature {
                    name: "create_sampler",
                    parameters: &[
                        FunctionParameter {
                            name: "device",
                            type_name: "& wgpu :: Device",
                        },
                        FunctionParameter {
                            name: "options",
                            type_name: "SamplerOptions",
                        },
                    ],
                    return_type: Some("wgpu :: Sampler"),
                },
            ],
        ),
        ("Space", &[]),
        (
            "Color",
            &[
                FunctionSignature {
                    name: "from_rgba",
                    parameters: &[FunctionParameter {
                        name: "d",
                        type_name: "[f32 ; 4]",
                    }],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "from_rgb_alpha",
                    parameters: &[
                        FunctionParameter {
                            name: "d",
                            type_name: "[f32 ; 3]",
                        },
                        FunctionParameter {
                            name: "alpha",
                            type_name: "f32",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "new",
                    parameters: &[
                        FunctionParameter {
                            name: "red",
                            type_name: "f32",
                        },
                        FunctionParameter {
                            name: "green",
                            type_name: "f32",
                        },
                        FunctionParameter {
                            name: "blue",
                            type_name: "f32",
                        },
                        FunctionParameter {
                            name: "alpha",
                            type_name: "f32",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "alpha",
                    parameters: &[],
                    return_type: Some("f32"),
                },
                FunctionSignature {
                    name: "into_vec4_gamma",
                    parameters: &[],
                    return_type: Some("[f32 ; 4]"),
                },
                FunctionSignature {
                    name: "red",
                    parameters: &[],
                    return_type: Some("f32"),
                },
                FunctionSignature {
                    name: "from_hex",
                    parameters: &[FunctionParameter {
                        name: "hex",
                        type_name: "& str",
                    }],
                    return_type: Some("Result < Self , csscolorparser :: ParseColorError >"),
                },
                FunctionSignature {
                    name: "from_css",
                    parameters: &[FunctionParameter {
                        name: "color",
                        type_name: "& str",
                    }],
                    return_type: Some("Result < Self , csscolorparser :: ParseColorError >"),
                },
                FunctionSignature {
                    name: "blue",
                    parameters: &[],
                    return_type: Some("f32"),
                },
                FunctionSignature {
                    name: "into_vec4",
                    parameters: &[],
                    return_type: Some("[f32 ; 4]"),
                },
                FunctionSignature {
                    name: "green",
                    parameters: &[],
                    return_type: Some("f32"),
                },
            ],
        ),
        (
            "MeshBuilder",
            &[
                FunctionSignature {
                    name: "vertex",
                    parameters: &[FunctionParameter {
                        name: "data",
                        type_name: "& [T]",
                    }],
                    return_type: Some("& 's mut Self"),
                },
                FunctionSignature {
                    name: "radius",
                    parameters: &[FunctionParameter {
                        name: "radius",
                        type_name: "f32",
                    }],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "renderer",
                        type_name: "& 'a mut renderer :: Renderer",
                    }],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "name",
                    parameters: &[FunctionParameter {
                        name: "name",
                        type_name: "& str",
                    }],
                    return_type: Some("& 's mut Self"),
                },
                FunctionSignature {
                    name: "build",
                    parameters: &[],
                    return_type: Some("Prototype"),
                },
                FunctionSignature {
                    name: "index",
                    parameters: &[FunctionParameter {
                        name: "data",
                        type_name: "& [u16]",
                    }],
                    return_type: Some("& 's mut Self"),
                },
            ],
        ),
        ("BufferPool", &[]),
        (
            "Texture",
            &[
                FunctionSignature {
                    name: "from_image",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& crate :: renderer :: Renderer",
                        },
                        FunctionParameter {
                            name: "image",
                            type_name: "& image :: DynamicImage",
                        },
                    ],
                    return_type: Some("Result < Self , Error >"),
                },
                FunctionSignature {
                    name: "from_wgpu_texture",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& crate :: renderer :: Renderer",
                        },
                        FunctionParameter {
                            name: "texture",
                            type_name: "wgpu :: Texture",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "write_data_to_texture",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& crate :: renderer :: Renderer",
                        },
                        FunctionParameter {
                            name: "origin_image",
                            type_name: "RgbaImage",
                        },
                        FunctionParameter {
                            name: "target_texture",
                            type_name: "& wgpu :: Texture",
                        },
                        FunctionParameter {
                            name: "size",
                            type_name: "wgpu :: Extent3d",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "create_target_texture",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& Renderer",
                        },
                        FunctionParameter {
                            name: "size",
                            type_name: "wgpu :: Extent3d",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "from_bytes",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& crate :: renderer :: Renderer",
                        },
                        FunctionParameter {
                            name: "bytes",
                            type_name: "& [u8]",
                        },
                    ],
                    return_type: Some("Result < Self , Error >"),
                },
                FunctionSignature {
                    name: "create_depth_texture",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& crate :: renderer :: Renderer",
                        },
                        FunctionParameter {
                            name: "size",
                            type_name: "wgpu :: Extent3d",
                        },
                    ],
                    return_type: Some("Self"),
                },
            ],
        ),
        ("TexCoords", &[]),
        (
            "TextureRegion",
            &[
                FunctionSignature {
                    name: "clamp_with_intersection",
                    parameters: &[
                        FunctionParameter {
                            name: "self_point",
                            type_name: "(i32 , i32)",
                        },
                        FunctionParameter {
                            name: "other_point",
                            type_name: "(i32 , i32)",
                        },
                        FunctionParameter {
                            name: "size",
                            type_name: "(i32 , i32)",
                        },
                        FunctionParameter {
                            name: "other",
                            type_name: "& mut TextureRegion",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "encompassing_pixels_i32",
                    parameters: &[
                        FunctionParameter {
                            name: "a",
                            type_name: "(i32 , i32)",
                        },
                        FunctionParameter {
                            name: "b",
                            type_name: "(i32 , i32)",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "for_pixel",
                    parameters: &[
                        FunctionParameter {
                            name: "x",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "y",
                            type_name: "u32",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "clamp",
                    parameters: &[
                        FunctionParameter {
                            name: "width",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "height",
                            type_name: "u32",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "for_whole_size",
                    parameters: &[
                        FunctionParameter {
                            name: "width",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "height",
                            type_name: "u32",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "intersects",
                    parameters: &[FunctionParameter {
                        name: "other",
                        type_name: "TextureRegion",
                    }],
                    return_type: Some("bool"),
                },
                FunctionSignature {
                    name: "for_region_i32",
                    parameters: &[
                        FunctionParameter {
                            name: "x",
                            type_name: "i32",
                        },
                        FunctionParameter {
                            name: "y",
                            type_name: "i32",
                        },
                        FunctionParameter {
                            name: "width",
                            type_name: "i32",
                        },
                        FunctionParameter {
                            name: "height",
                            type_name: "i32",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "for_region",
                    parameters: &[
                        FunctionParameter {
                            name: "x",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "y",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "width",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "height",
                            type_name: "u32",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "encompassing_pixels",
                    parameters: &[
                        FunctionParameter {
                            name: "a",
                            type_name: "(u32 , u32)",
                        },
                        FunctionParameter {
                            name: "b",
                            type_name: "(u32 , u32)",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "encompass",
                    parameters: &[
                        FunctionParameter {
                            name: "x",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "y",
                            type_name: "u32",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "height",
                    parameters: &[],
                    return_type: Some("u32"),
                },
                FunctionSignature {
                    name: "width",
                    parameters: &[],
                    return_type: Some("u32"),
                },
                FunctionSignature {
                    name: "union",
                    parameters: &[FunctionParameter {
                        name: "other",
                        type_name: "TextureRegion",
                    }],
                    return_type: None,
                },
            ],
        ),
        (
            "asset_obj.rs",
            &[FunctionSignature {
                name: "load_obj",
                parameters: &[
                    FunctionParameter {
                        name: "path",
                        type_name: "impl AsRef < Path >",
                    },
                    FunctionParameter {
                        name: "scene",
                        type_name: "& mut crate :: Scene",
                    },
                    FunctionParameter {
                        name: "node",
                        type_name: "crate :: NodeId",
                    },
                    FunctionParameter {
                        name: "context",
                        type_name: "& mut crate :: Renderer",
                    },
                ],
                return_type: Some(
                    "fxhash :: FxHashMap < String , (crate :: EntityRef , crate :: Prototype) >",
                ),
            }],
        ),
        (
            "Window",
            &[
                FunctionSignature {
                    name: "new",
                    parameters: &[],
                    return_type: Some("WindowBuilder"),
                },
                FunctionSignature {
                    name: "run",
                    parameters: &[FunctionParameter {
                        name: "mut runner",
                        type_name: "impl 'static + FnMut (Event)",
                    }],
                    return_type: Some("!"),
                },
            ],
        ),
        (
            "Scene",
            &[
                FunctionSignature {
                    name: "new",
                    parameters: &[],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "add_directional_light",
                    parameters: &[],
                    return_type: Some("ObjectBuilder < LightBuilder >"),
                },
                FunctionSignature {
                    name: "add_point_light",
                    parameters: &[],
                    return_type: Some("ObjectBuilder < LightBuilder >"),
                },
                FunctionSignature {
                    name: "active_camera",
                    parameters: &[],
                    return_type: None,
                },
                FunctionSignature {
                    name: "lights",
                    parameters: &[],
                    return_type: Some("impl Iterator < Item = (LightRef , & 'a Light) >"),
                },
                FunctionSignature {
                    name: "add",
                    parameters: &[FunctionParameter {
                        name: "_components",
                        type_name: "impl hecs :: DynamicBundle",
                    }],
                    return_type: Some("hecs :: Entity"),
                },
                FunctionSignature {
                    name: "bake",
                    parameters: &[],
                    return_type: Some("BakedScene"),
                },
                FunctionSignature {
                    name: "add_entity",
                    parameters: &[FunctionParameter {
                        name: "prototype",
                        type_name: "& Prototype",
                    }],
                    return_type: Some("ObjectBuilder < EntityBuilder >"),
                },
                FunctionSignature {
                    name: "add_sprite",
                    parameters: &[FunctionParameter {
                        name: "image",
                        type_name: "TextureRef",
                    }],
                    return_type: Some("ObjectBuilder < SpriteBuilder >"),
                },
                FunctionSignature {
                    name: "add_node",
                    parameters: &[],
                    return_type: Some("ObjectBuilder < () >"),
                },
                FunctionSignature {
                    name: "add_light",
                    parameters: &[FunctionParameter {
                        name: "kind",
                        type_name: "LightKind",
                    }],
                    return_type: Some("ObjectBuilder < LightBuilder >"),
                },
            ],
        ),
        (
            "TextureTarget",
            &[
                FunctionSignature {
                    name: "from_wgpu_texture",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& Renderer",
                        },
                        FunctionParameter {
                            name: "texture",
                            type_name: "wgpu :: Texture",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "from_texture",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& Renderer",
                        },
                        FunctionParameter {
                            name: "texture",
                            type_name: "Texture",
                        },
                    ],
                    return_type: Some("Result < Self , Error >"),
                },
                FunctionSignature {
                    name: "new",
                    parameters: &[
                        FunctionParameter {
                            name: "renderer",
                            type_name: "& Renderer",
                        },
                        FunctionParameter {
                            name: "size",
                            type_name: "wgpu :: Extent3d",
                        },
                    ],
                    return_type: Some("Result < Self , Error >"),
                },
            ],
        ),
        ("Target", &[]),
        (
            "BufferSize",
            &[
                FunctionSignature {
                    name: "size",
                    parameters: &[],
                    return_type: Some("u64"),
                },
                FunctionSignature {
                    name: "new",
                    parameters: &[
                        FunctionParameter {
                            name: "width",
                            type_name: "usize",
                        },
                        FunctionParameter {
                            name: "height",
                            type_name: "usize",
                        },
                    ],
                    return_type: Some("Self"),
                },
            ],
        ),
        (
            "WindowBuilder",
            &[
                FunctionSignature {
                    name: "title",
                    parameters: &[FunctionParameter {
                        name: "title",
                        type_name: "& str",
                    }],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "size",
                    parameters: &[
                        FunctionParameter {
                            name: "width",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "height",
                            type_name: "u32",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "build",
                    parameters: &[],
                    return_type: Some("Window"),
                },
            ],
        ),
        (
            "SpriteMap",
            &[FunctionSignature {
                name: "at",
                parameters: &[FunctionParameter {
                    name: "index",
                    type_name: "mint :: Point2 < usize >",
                }],
                return_type: Some("crate :: UvRange"),
            }],
        ),
        ("Position", &[]),
        ("Material", &[]),
        (
            "Solid",
            &[
                FunctionSignature {
                    name: "new",
                    parameters: &[
                        FunctionParameter {
                            name: "config",
                            type_name: "& SolidConfig",
                        },
                        FunctionParameter {
                            name: "context",
                            type_name: "& crate :: Renderer",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "new_offscreen",
                    parameters: &[
                        FunctionParameter {
                            name: "config",
                            type_name: "& SolidConfig",
                        },
                        FunctionParameter {
                            name: "target_info",
                            type_name: "crate :: TargetInfo",
                        },
                        FunctionParameter {
                            name: "context",
                            type_name: "& crate :: Renderer",
                        },
                    ],
                    return_type: Some("Self"),
                },
            ],
        ),
        (
            "Phong",
            &[
                FunctionSignature {
                    name: "new_offscreen",
                    parameters: &[
                        FunctionParameter {
                            name: "config",
                            type_name: "& PhongConfig",
                        },
                        FunctionParameter {
                            name: "target_info",
                            type_name: "crate :: TargetInfo",
                        },
                        FunctionParameter {
                            name: "context",
                            type_name: "& crate :: Renderer",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "new",
                    parameters: &[
                        FunctionParameter {
                            name: "config",
                            type_name: "& PhongConfig",
                        },
                        FunctionParameter {
                            name: "context",
                            type_name: "& crate :: Renderer",
                        },
                    ],
                    return_type: Some("Self"),
                },
            ],
        ),
        (
            "Renderer",
            &[
                FunctionSignature {
                    name: "add_mesh",
                    parameters: &[],
                    return_type: Some("MeshBuilder"),
                },
                FunctionSignature {
                    name: "add_texture",
                    parameters: &[
                        FunctionParameter {
                            name: "texture",
                            type_name: "wgpu :: Texture",
                        },
                        FunctionParameter {
                            name: "size",
                            type_name: "wgpu :: Extent3d",
                        },
                    ],
                    return_type: Some("TextureRef"),
                },
                FunctionSignature {
                    name: "add_image_from_bytes",
                    parameters: &[
                        FunctionParameter {
                            name: "desc",
                            type_name: "& wgpu :: TextureDescriptor",
                        },
                        FunctionParameter {
                            name: "data",
                            type_name: "& [u8]",
                        },
                    ],
                    return_type: Some("TextureRef"),
                },
                FunctionSignature {
                    name: "surface_info",
                    parameters: &[],
                    return_type: Some("Option < TargetInfo >"),
                },
                FunctionSignature {
                    name: "resize",
                    parameters: &[
                        FunctionParameter {
                            name: "width",
                            type_name: "u32",
                        },
                        FunctionParameter {
                            name: "height",
                            type_name: "u32",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "load_image",
                    parameters: &[FunctionParameter {
                        name: "path_ref",
                        type_name: "impl AsRef < Path >",
                    }],
                    return_type: Some("TextureRef"),
                },
                FunctionSignature {
                    name: "init",
                    parameters: &[],
                    return_type: Some("RendererBuilder"),
                },
                FunctionSignature {
                    name: "present",
                    parameters: &[
                        FunctionParameter {
                            name: "pass",
                            type_name: "& mut P",
                        },
                        FunctionParameter {
                            name: "scene",
                            type_name: "& Scene",
                        },
                        FunctionParameter {
                            name: "camera",
                            type_name: "& Camera",
                        },
                    ],
                    return_type: Some("Result < () , Error >"),
                },
            ],
        ),
        (
            "Camera",
            &[FunctionSignature {
                name: "projection_matrix",
                parameters: &[FunctionParameter {
                    name: "aspect",
                    type_name: "f32",
                }],
                return_type: Some("mint :: ColumnMatrix4 < f32 >"),
            }],
        ),
        ("SolidConfig", &[]),
        ("SamplerOptions", &[]),
        (
            "ObjectBuilder",
            &[
                FunctionSignature {
                    name: "orientation_around",
                    parameters: &[
                        FunctionParameter {
                            name: "axis",
                            type_name: "mint :: Vector3 < f32 >",
                        },
                        FunctionParameter {
                            name: "angle_deg",
                            type_name: "f32",
                        },
                    ],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "build",
                    parameters: &[],
                    return_type: Some("NodeId"),
                },
                FunctionSignature {
                    name: "component",
                    parameters: &[FunctionParameter {
                        name: "component",
                        type_name: "T",
                    }],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "orientation",
                    parameters: &[FunctionParameter {
                        name: "quat",
                        type_name: "mint :: Quaternion < f32 >",
                    }],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "look_at",
                    parameters: &[
                        FunctionParameter {
                            name: "target",
                            type_name: "mint :: Vector3 < f32 >",
                        },
                        FunctionParameter {
                            name: "up",
                            type_name: "mint :: Vector3 < f32 >",
                        },
                    ],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "parent",
                    parameters: &[FunctionParameter {
                        name: "parent",
                        type_name: "NodeId",
                    }],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "position",
                    parameters: &[FunctionParameter {
                        name: "position",
                        type_name: "mint :: Vector3 < f32 >",
                    }],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "color",
                    parameters: &[FunctionParameter {
                        name: "color",
                        type_name: "Color",
                    }],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "uv",
                    parameters: &[FunctionParameter {
                        name: "uv",
                        type_name: "UvRange",
                    }],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "scale",
                    parameters: &[FunctionParameter {
                        name: "scale",
                        type_name: "f32",
                    }],
                    return_type: Some("& mut Self"),
                },
                FunctionSignature {
                    name: "intensity",
                    parameters: &[FunctionParameter {
                        name: "intensity",
                        type_name: "f32",
                    }],
                    return_type: Some("& mut Self"),
                },
            ],
        ),
        (
            "Real",
            &[
                FunctionSignature {
                    name: "new",
                    parameters: &[
                        FunctionParameter {
                            name: "config",
                            type_name: "& RealConfig",
                        },
                        FunctionParameter {
                            name: "context",
                            type_name: "& crate :: Renderer",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "new_offscreen",
                    parameters: &[
                        FunctionParameter {
                            name: "config",
                            type_name: "& RealConfig",
                        },
                        FunctionParameter {
                            name: "target_info",
                            type_name: "crate :: TargetInfo",
                        },
                        FunctionParameter {
                            name: "context",
                            type_name: "& crate :: Renderer",
                        },
                    ],
                    return_type: Some("Self"),
                },
            ],
        ),
        ("Normal", &[]),
        (
            "Node",
            &[
                FunctionSignature {
                    name: "set_scale",
                    parameters: &[FunctionParameter {
                        name: "scale",
                        type_name: "f32",
                    }],
                    return_type: None,
                },
                FunctionSignature {
                    name: "set_position",
                    parameters: &[FunctionParameter {
                        name: "pos",
                        type_name: "mint :: Vector3 < f32 >",
                    }],
                    return_type: None,
                },
                FunctionSignature {
                    name: "get_position",
                    parameters: &[],
                    return_type: Some("mint :: Vector3 < f32 >"),
                },
                FunctionSignature {
                    name: "rotate",
                    parameters: &[
                        FunctionParameter {
                            name: "axis",
                            type_name: "mint :: Vector3 < f32 >",
                        },
                        FunctionParameter {
                            name: "angle_deg",
                            type_name: "f32",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "pre_rotate",
                    parameters: &[
                        FunctionParameter {
                            name: "axis",
                            type_name: "mint :: Vector3 < f32 >",
                        },
                        FunctionParameter {
                            name: "angle_deg",
                            type_name: "f32",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "get_rotation",
                    parameters: &[],
                    return_type: Some("(mint :: Vector3 < f32 > , f32)"),
                },
                FunctionSignature {
                    name: "post_rotate",
                    parameters: &[
                        FunctionParameter {
                            name: "axis",
                            type_name: "mint :: Vector3 < f32 >",
                        },
                        FunctionParameter {
                            name: "angle_deg",
                            type_name: "f32",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "get_scale",
                    parameters: &[],
                    return_type: Some("f32"),
                },
                FunctionSignature {
                    name: "post_move",
                    parameters: &[FunctionParameter {
                        name: "offset",
                        type_name: "mint :: Vector3 < f32 >",
                    }],
                    return_type: None,
                },
                FunctionSignature {
                    name: "set_rotation",
                    parameters: &[
                        FunctionParameter {
                            name: "axis",
                            type_name: "mint :: Vector3 < f32 >",
                        },
                        FunctionParameter {
                            name: "angle_deg",
                            type_name: "f32",
                        },
                    ],
                    return_type: None,
                },
                FunctionSignature {
                    name: "r#move",
                    parameters: &[FunctionParameter {
                        name: "offset",
                        type_name: "mint :: Vector3 < f32 >",
                    }],
                    return_type: None,
                },
                FunctionSignature {
                    name: "pre_move",
                    parameters: &[FunctionParameter {
                        name: "offset",
                        type_name: "mint :: Vector3 < f32 >",
                    }],
                    return_type: None,
                },
            ],
        ),
        ("Ambient", &[]),
        (
            "Mesh",
            &[
                FunctionSignature {
                    name: "vertex_stream",
                    parameters: &[],
                    return_type: Some("Option < & VertexStream >"),
                },
                FunctionSignature {
                    name: "vertex_slice",
                    parameters: &[],
                    return_type: Some("wgpu :: BufferSlice"),
                },
            ],
        ),
        (
            "Geometry",
            &[
                FunctionSignature {
                    name: "bake",
                    parameters: &[FunctionParameter {
                        name: "context",
                        type_name: "& mut plr :: Renderer",
                    }],
                    return_type: Some("plr :: Prototype"),
                },
                FunctionSignature {
                    name: "fill",
                    parameters: &[FunctionParameter {
                        name: "path",
                        type_name: "& Path",
                    }],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "sphere",
                    parameters: &[
                        FunctionParameter {
                            name: "streams",
                            type_name: "super :: Streams",
                        },
                        FunctionParameter {
                            name: "radius",
                            type_name: "f32",
                        },
                        FunctionParameter {
                            name: "detail",
                            type_name: "usize",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "cuboid",
                    parameters: &[
                        FunctionParameter {
                            name: "streams",
                            type_name: "super :: Streams",
                        },
                        FunctionParameter {
                            name: "half_extent",
                            type_name: "mint :: Vector3 < f32 >",
                        },
                    ],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "plane",
                    parameters: &[FunctionParameter {
                        name: "size",
                        type_name: "f32",
                    }],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "stroke",
                    parameters: &[
                        FunctionParameter {
                            name: "path",
                            type_name: "& Path",
                        },
                        FunctionParameter {
                            name: "options",
                            type_name: "& StrokeOptions",
                        },
                    ],
                    return_type: Some("Self"),
                },
            ],
        ),
        (
            "asset_gltf.rs",
            &[FunctionSignature {
                name: "load_gltf",
                parameters: &[
                    FunctionParameter {
                        name: "path",
                        type_name: "impl AsRef < Path >",
                    },
                    FunctionParameter {
                        name: "scene",
                        type_name: "& mut crate :: Scene",
                    },
                    FunctionParameter {
                        name: "global_parent",
                        type_name: "crate :: NodeId",
                    },
                    FunctionParameter {
                        name: "context",
                        type_name: "& mut crate :: Renderer",
                    },
                ],
                return_type: Some("Module"),
            }],
        ),
        (
            "Flat",
            &[
                FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "context",
                        type_name: "& crate :: Renderer",
                    }],
                    return_type: Some("Self"),
                },
                FunctionSignature {
                    name: "new_offscreen",
                    parameters: &[
                        FunctionParameter {
                            name: "target_info",
                            type_name: "crate :: TargetInfo",
                        },
                        FunctionParameter {
                            name: "context",
                            type_name: "& crate :: Renderer",
                        },
                    ],
                    return_type: Some("Self"),
                },
            ],
        ),
        ("BakedScene", &[]),
        (
            "RawSpace",
            &[
                FunctionSignature {
                    name: "to_space",
                    parameters: &[],
                    return_type: Some("Space"),
                },
                FunctionSignature {
                    name: "inverse_matrix",
                    parameters: &[],
                    return_type: Some("mint :: ColumnMatrix4 < f32 >"),
                },
            ],
        ),
        ("Array", &[]),
        ("RealConfig", &[]),
    ],
};
