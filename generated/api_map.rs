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

#[derive(Clone, Debug, PartialEq)]
struct ObjectProperty {
    pub name: &'static str,
    pub type_name: &'static str,
    pub function: Option<FunctionSignature>,
}

static API_MAP: phf::Map<&'static str, &[ObjectProperty]> = ::phf::Map {
    key: 12913932095322966823,
    disps: &[
        (0, 0),
        (2, 11),
        (0, 9),
        (0, 76),
        (1, 62),
        (0, 0),
        (0, 6),
        (0, 47),
        (0, 10),
        (2, 28),
        (4, 30),
        (0, 18),
        (0, 57),
        (1, 0),
        (17, 46),
        (4, 49),
    ],
    entries: &[
        ("RenderTarget", &[]),
        (
            "RenderTargets",
            &[
                ObjectProperty {
                    name: "targets",
                    type_name: "HashMap < TargetId , RenderTarget >",
                    function: None,
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[],
                        return_type: Some("Self"),
                    }),
                },
            ],
        ),
        ("Vec2or3", &[]),
        (
            "WindowState",
            &[
                ObjectProperty {
                    name: "get_hovered_file",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "get_hovered_file",
                        parameters: &[FunctionParameter {
                            name: "index",
                            type_name: "u128",
                        }],
                        return_type: Some("Option < String >"),
                    }),
                },
                ObjectProperty {
                    name: "close_on_esc",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "get_dropped_file",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "get_dropped_file",
                        parameters: &[FunctionParameter {
                            name: "index",
                            type_name: "u128",
                        }],
                        return_type: Some("Option < PathBuf >"),
                    }),
                },
                ObjectProperty {
                    name: "target_frametime",
                    type_name: "Option < f64 >",
                    function: None,
                },
                ObjectProperty {
                    name: "auto_resize",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "hovered_files",
                    type_name: "HashMap < u128 , PathBuf >",
                    function: None,
                },
                ObjectProperty {
                    name: "redraw",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "redraw",
                        parameters: &[],
                        return_type: None,
                    }),
                },
            ],
        ),
        (
            "Sphere",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[
                        FunctionParameter {
                            name: "radius",
                            type_name: "f32",
                        },
                        FunctionParameter {
                            name: "detail",
                            type_name: "usize",
                        },
                    ],
                    return_type: Some("Object < Mesh >"),
                }),
            }],
        ),
        (
            "Resources",
            &[
                ObjectProperty {
                    name: "get_texture",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "get_texture",
                        parameters: &[FunctionParameter {
                            name: "id",
                            type_name: "& TextureId",
                        }],
                        return_type: Some("Option < & Texture >"),
                    }),
                },
                ObjectProperty {
                    name: "add_mesh",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "add_mesh",
                        parameters: &[FunctionParameter {
                            name: "mesh",
                            type_name: "MeshData",
                        }],
                        return_type: Some("MeshId"),
                    }),
                },
                ObjectProperty {
                    name: "remove_mesh",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "remove_mesh",
                        parameters: &[FunctionParameter {
                            name: "id",
                            type_name: "& MeshId",
                        }],
                        return_type: Some("Option < MeshData >"),
                    }),
                },
                ObjectProperty {
                    name: "get_mesh",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "get_mesh",
                        parameters: &[FunctionParameter {
                            name: "id",
                            type_name: "& MeshId",
                        }],
                        return_type: Some("Option < & MeshData >"),
                    }),
                },
                ObjectProperty {
                    name: "add_texture",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "add_texture",
                        parameters: &[FunctionParameter {
                            name: "texture",
                            type_name: "Texture",
                        }],
                        return_type: Some("TextureId"),
                    }),
                },
                ObjectProperty {
                    name: "remove_texture",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "remove_texture",
                        parameters: &[FunctionParameter {
                            name: "id",
                            type_name: "& TextureId",
                        }],
                        return_type: Some("Option < Texture >"),
                    }),
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[],
                        return_type: Some("Self"),
                    }),
                },
            ],
        ),
        ("LightType", &[]),
        (
            "Shader",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "source",
                        type_name: "& str",
                    }],
                    return_type: Some("Object < Self >"),
                }),
            }],
        ),
        (
            "Light",
            &[
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[FunctionParameter {
                            name: "options",
                            type_name: "LightOptions",
                        }],
                        return_type: Some("Object < Self >"),
                    }),
                },
                ObjectProperty {
                    name: "color",
                    type_name: "Color",
                    function: None,
                },
                ObjectProperty {
                    name: "intensity",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "variant",
                    type_name: "LightType",
                    function: None,
                },
            ],
        ),
        (
            "Mesh",
            &[
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[FunctionParameter {
                            name: "built_mesh",
                            type_name: "Option < BuiltMesh >",
                        }],
                        return_type: Some("Object < Self >"),
                    }),
                },
                ObjectProperty {
                    name: "mesh_id",
                    type_name: "MeshId",
                    function: None,
                },
            ],
        ),
        ("ProjectionOptions", &[]),
        ("Vec", &[]),
        (
            "Line",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[
                        FunctionParameter {
                            name: "from",
                            type_name: "Pixel",
                        },
                        FunctionParameter {
                            name: "to",
                            type_name: "Pixel",
                        },
                    ],
                    return_type: Some("Object < Shape >"),
                }),
            }],
        ),
        ("ShapeType", &[]),
        (
            "Primitive",
            &[
                ObjectProperty {
                    name: "cuboid",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "cuboid",
                        parameters: &[FunctionParameter {
                            name: "dimensions",
                            type_name: "V",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "indices",
                    type_name: "Option < Vec < u16 > >",
                    function: None,
                },
                ObjectProperty {
                    name: "normals",
                    type_name: "Option < Vec < vertex :: Normal > >",
                    function: None,
                },
                ObjectProperty {
                    name: "positions",
                    type_name: "Vec < vertex :: Position >",
                    function: None,
                },
                ObjectProperty {
                    name: "plane",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "plane",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "f32",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "sphere",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "sphere",
                        parameters: &[
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
                    }),
                },
                ObjectProperty {
                    name: "create_mesh",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "create_mesh",
                        parameters: &[],
                        return_type: Some("Result < mesh :: BuiltMesh , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "radius",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "cube",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "cube",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "f32",
                        }],
                        return_type: Some("Self"),
                    }),
                },
            ],
        ),
        (
            "Quad",
            &[
                ObjectProperty {
                    name: "smaller_side",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "smaller_side",
                        parameters: &[],
                        return_type: Some("u32"),
                    }),
                },
                ObjectProperty {
                    name: "inbound_radius",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "inbound_radius",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "min_x",
                    type_name: "u32",
                    function: None,
                },
                ObjectProperty {
                    name: "max_x",
                    type_name: "u32",
                    function: None,
                },
                ObjectProperty {
                    name: "to_vec2",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "to_vec2",
                        parameters: &[],
                        return_type: Some("Vec2"),
                    }),
                },
                ObjectProperty {
                    name: "center_f32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "center_f32",
                        parameters: &[],
                        return_type: Some("Vec2"),
                    }),
                },
                ObjectProperty {
                    name: "aspect",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "aspect",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "from_size_f32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_size_f32",
                        parameters: &[
                            FunctionParameter {
                                name: "width",
                                type_name: "f32",
                            },
                            FunctionParameter {
                                name: "height",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "pixel_center",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pixel_center",
                        parameters: &[],
                        return_type: Some("(u32 , u32)"),
                    }),
                },
                ObjectProperty {
                    name: "clamp_with_intersection",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
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
                                type_name: "& mut Quad",
                            },
                        ],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "is_larger_than",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "is_larger_than",
                        parameters: &[FunctionParameter {
                            name: "other",
                            type_name: "Quad",
                        }],
                        return_type: Some("bool"),
                    }),
                },
                ObjectProperty {
                    name: "equals",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "equals",
                        parameters: &[FunctionParameter {
                            name: "other",
                            type_name: "Quad",
                        }],
                        return_type: Some("bool"),
                    }),
                },
                ObjectProperty {
                    name: "from_window_logical_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_window_logical_size",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "& winit :: dpi :: LogicalSize < u32 >",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "from_region",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_region",
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
                    }),
                },
                ObjectProperty {
                    name: "from_tuples",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_tuples",
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
                    }),
                },
                ObjectProperty {
                    name: "width",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "width",
                        parameters: &[],
                        return_type: Some("u32"),
                    }),
                },
                ObjectProperty {
                    name: "half_width_f32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "half_width_f32",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "half_width",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "half_width",
                        parameters: &[],
                        return_type: Some("u32"),
                    }),
                },
                ObjectProperty {
                    name: "to_vec4",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "to_vec4",
                        parameters: &[],
                        return_type: Some("Vec4"),
                    }),
                },
                ObjectProperty {
                    name: "half_height_f32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "half_height_f32",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "to_wgpu_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "to_wgpu_size",
                        parameters: &[],
                        return_type: Some("wgpu :: Extent3d"),
                    }),
                },
                ObjectProperty {
                    name: "clamp",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
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
                    }),
                },
                ObjectProperty {
                    name: "width_f32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "width_f32",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "larger_side",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "larger_side",
                        parameters: &[],
                        return_type: Some("u32"),
                    }),
                },
                ObjectProperty {
                    name: "height_f32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "height_f32",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "union",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "union",
                        parameters: &[FunctionParameter {
                            name: "other",
                            type_name: "Quad",
                        }],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "area",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "area",
                        parameters: &[],
                        return_type: Some("u32"),
                    }),
                },
                ObjectProperty {
                    name: "antialias_factor",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "antialias_factor",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "min_y",
                    type_name: "u32",
                    function: None,
                },
                ObjectProperty {
                    name: "from_tuple",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_tuple",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "(u32 , u32)",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "from_inbound_radius",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_inbound_radius",
                        parameters: &[FunctionParameter {
                            name: "radius",
                            type_name: "f32",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "max_y",
                    type_name: "u32",
                    function: None,
                },
                ObjectProperty {
                    name: "to_range",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "to_range",
                        parameters: &[],
                        return_type: Some("std :: ops :: Range < mint :: Point2 < i32 > >"),
                    }),
                },
                ObjectProperty {
                    name: "intersects",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "intersects",
                        parameters: &[FunctionParameter {
                            name: "other",
                            type_name: "Quad",
                        }],
                        return_type: Some("bool"),
                    }),
                },
                ObjectProperty {
                    name: "from_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_size",
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
                    }),
                },
                ObjectProperty {
                    name: "from_window_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_window_size",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "& winit :: dpi :: PhysicalSize < u32 >",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "from_region_i32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_region_i32",
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
                    }),
                },
                ObjectProperty {
                    name: "from_wgpu_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_wgpu_size",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "wgpu :: Extent3d",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "to_array",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "to_array",
                        parameters: &[],
                        return_type: Some("[f32 ; 4]"),
                    }),
                },
                ObjectProperty {
                    name: "from_arrays_i32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_arrays_i32",
                        parameters: &[
                            FunctionParameter {
                                name: "a",
                                type_name: "[i32 ; 2]",
                            },
                            FunctionParameter {
                                name: "b",
                                type_name: "[i32 ; 2]",
                            },
                        ],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "from_pixel",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_pixel",
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
                    }),
                },
                ObjectProperty {
                    name: "half_height",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "half_height",
                        parameters: &[],
                        return_type: Some("u32"),
                    }),
                },
                ObjectProperty {
                    name: "outbound_radius",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "outbound_radius",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "height",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "height",
                        parameters: &[],
                        return_type: Some("u32"),
                    }),
                },
                ObjectProperty {
                    name: "encompass",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
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
                    }),
                },
                ObjectProperty {
                    name: "is_smaller_than",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "is_smaller_than",
                        parameters: &[FunctionParameter {
                            name: "other",
                            type_name: "Quad",
                        }],
                        return_type: Some("bool"),
                    }),
                },
                ObjectProperty {
                    name: "from_tuples_i32",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_tuples_i32",
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
                    }),
                },
            ],
        ),
        (
            "SceneId",
            &[ObjectProperty {
                name: "",
                type_name: "u32",
                function: None,
            }],
        ),
        (
            "Projection",
            &[
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[FunctionParameter {
                            name: "options",
                            type_name: "ProjectionOptions",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "from_target_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_target_size",
                        parameters: &[FunctionParameter {
                            name: "quad",
                            type_name: "Quad",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "perspective",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "perspective",
                        parameters: &[FunctionParameter {
                            name: "fov_y",
                            type_name: "f32",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "orthographic",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "orthographic",
                        parameters: &[
                            FunctionParameter {
                                name: "center",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "size",
                                type_name: "Quad",
                            },
                        ],
                        return_type: Some("Self"),
                    }),
                },
            ],
        ),
        (
            "RendererOptions",
            &[
                ObjectProperty {
                    name: "power_preference",
                    type_name: "String",
                    function: None,
                },
                ObjectProperty {
                    name: "device_limits",
                    type_name: "String",
                    function: None,
                },
                ObjectProperty {
                    name: "force_software_rendering",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "render_pass",
                    type_name: "String",
                    function: None,
                },
                ObjectProperty {
                    name: "panic_on_error",
                    type_name: "bool",
                    function: None,
                },
            ],
        ),
        (
            "Object",
            &[
                ObjectProperty {
                    name: "set_image",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_image",
                        parameters: &[FunctionParameter {
                            name: "bytes",
                            type_name: "& [u8]",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "read_component",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "read_component",
                        parameters: &[],
                        return_type: Some("Option < C >"),
                    }),
                },
                ObjectProperty {
                    name: "set_clip_region",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_clip_region",
                        parameters: &[FunctionParameter {
                            name: "clip_region",
                            type_name: "Quad",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "add_components",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "add_components",
                        parameters: &[FunctionParameter {
                            name: "bundle",
                            type_name: "B",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotation_degrees",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotation_degrees",
                        parameters: &[],
                        return_type: Some("(Vec3 , f32)"),
                    }),
                },
                ObjectProperty {
                    name: "set_mesh",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_mesh",
                        parameters: &[FunctionParameter {
                            name: "built_mesh",
                            type_name: "Option < BuiltMesh >",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "pre_rotate_degrees",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pre_rotate_degrees",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "thickness",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "thickness",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "set_intensity",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_intensity",
                        parameters: &[FunctionParameter {
                            name: "intensity",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "update_components",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "update_components",
                        parameters: &[FunctionParameter {
                            name: "bundle",
                            type_name: "B",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotation_quaternion",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotation_quaternion",
                        parameters: &[],
                        return_type: Some("Quaternion"),
                    }),
                },
                ObjectProperty {
                    name: "radius",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "radius",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "to",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "to",
                        parameters: &[],
                        return_type: Some("Pixel"),
                    }),
                },
                ObjectProperty {
                    name: "set_position",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_position",
                        parameters: &[FunctionParameter {
                            name: "position",
                            type_name: "V",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "pre_rotate_radians",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pre_rotate_radians",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "radians",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "hide",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "hide",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "update_component",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "update_component",
                        parameters: &[FunctionParameter {
                            name: "component",
                            type_name: "C",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_border",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_border",
                        parameters: &[FunctionParameter {
                            name: "border",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "image",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "image",
                        parameters: &[],
                        return_type: Some("TextureId"),
                    }),
                },
                ObjectProperty {
                    name: "set_shape_type",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_shape_type",
                        parameters: &[FunctionParameter {
                            name: "shape",
                            type_name: "ShapeType",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "has_moved",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "has_moved",
                        parameters: &[],
                        return_type: Some("bool"),
                    }),
                },
                ObjectProperty {
                    name: "height",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "height",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "set_height",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_height",
                        parameters: &[FunctionParameter {
                            name: "height",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_to",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_to",
                        parameters: &[FunctionParameter {
                            name: "to",
                            type_name: "Pixel",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_parent",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_parent",
                        parameters: &[FunctionParameter {
                            name: "parent",
                            type_name: "& impl SceneObject",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_width",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_width",
                        parameters: &[FunctionParameter {
                            name: "width",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_rotation_quaternion",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_rotation_quaternion",
                        parameters: &[FunctionParameter {
                            name: "quat",
                            type_name: "Q",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "add_component",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "add_component",
                        parameters: &[FunctionParameter {
                            name: "component",
                            type_name: "C",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_parent_transform",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_parent_transform",
                        parameters: &[FunctionParameter {
                            name: "parent",
                            type_name: "TransformId",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_color",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_color",
                        parameters: &[FunctionParameter {
                            name: "color",
                            type_name: "Color",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotate_degrees",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotate_degrees",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotate_radians",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotate_radians",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "radians",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "pre_rotate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pre_rotate",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "color",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "color",
                        parameters: &[],
                        return_type: Some("Color"),
                    }),
                },
                ObjectProperty {
                    name: "upsert_component",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "upsert_component",
                        parameters: &[FunctionParameter {
                            name: "component",
                            type_name: "C",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_rotation",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_rotation",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "look_at_origin",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "look_at_origin",
                        parameters: &[FunctionParameter {
                            name: "up",
                            type_name: "V",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "remove_components",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "remove_components",
                        parameters: &[],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "upsert_components",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "upsert_components",
                        parameters: &[FunctionParameter {
                            name: "bundle",
                            type_name: "B",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "shape_type",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "shape_type",
                        parameters: &[],
                        return_type: Some("ShapeType"),
                    }),
                },
                ObjectProperty {
                    name: "look_at",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "look_at",
                        parameters: &[
                            FunctionParameter {
                                name: "target",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "up",
                                type_name: "V",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "local_transform",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "local_transform",
                        parameters: &[],
                        return_type: Some("LocalTransform"),
                    }),
                },
                ObjectProperty {
                    name: "apply",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "apply",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "mesh",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "mesh",
                        parameters: &[],
                        return_type: Some("MeshId"),
                    }),
                },
                ObjectProperty {
                    name: "set_rotation_degrees",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_rotation_degrees",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_thickness",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_thickness",
                        parameters: &[FunctionParameter {
                            name: "thickness",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "remove_component",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "remove_component",
                        parameters: &[],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotation",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotation",
                        parameters: &[],
                        return_type: Some("(Vec3 , f32)"),
                    }),
                },
                ObjectProperty {
                    name: "clip_region",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "clip_region",
                        parameters: &[],
                        return_type: Some("Option < Quad >"),
                    }),
                },
                ObjectProperty {
                    name: "rotate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotate",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "show",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "show",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "load_image",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "load_image",
                        parameters: &[FunctionParameter {
                            name: "image_path",
                            type_name: "impl AsRef < Path >",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "border",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "border",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "set_radius",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_radius",
                        parameters: &[FunctionParameter {
                            name: "radius",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "pre_translate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pre_translate",
                        parameters: &[FunctionParameter {
                            name: "offset",
                            type_name: "V",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "has_component",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "has_component",
                        parameters: &[],
                        return_type: Some("bool"),
                    }),
                },
                ObjectProperty {
                    name: "parent",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "parent",
                        parameters: &[],
                        return_type: Some("TransformId"),
                    }),
                },
                ObjectProperty {
                    name: "rotation_radians",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotation_radians",
                        parameters: &[],
                        return_type: Some("(Vec3 , f32)"),
                    }),
                },
                ObjectProperty {
                    name: "set_rotation_radians",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_rotation_radians",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "radians",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "width",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "width",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "from",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from",
                        parameters: &[],
                        return_type: Some("Pixel"),
                    }),
                },
                ObjectProperty {
                    name: "set_scale",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_scale",
                        parameters: &[FunctionParameter {
                            name: "scale",
                            type_name: "S",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "batch",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "batch",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "translate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "translate",
                        parameters: &[FunctionParameter {
                            name: "offset",
                            type_name: "V",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[FunctionParameter {
                            name: "object",
                            type_name: "T",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "position",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "position",
                        parameters: &[],
                        return_type: Some("Vec3"),
                    }),
                },
                ObjectProperty {
                    name: "scale",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "scale",
                        parameters: &[],
                        return_type: Some("Vec3"),
                    }),
                },
                ObjectProperty {
                    name: "set_from",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_from",
                        parameters: &[FunctionParameter {
                            name: "from",
                            type_name: "Pixel",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
            ],
        ),
        (
            "VertexData",
            &[
                ObjectProperty {
                    name: "stride",
                    type_name: "wgpu :: BufferAddress",
                    function: None,
                },
                ObjectProperty {
                    name: "offset",
                    type_name: "wgpu :: BufferAddress",
                    function: None,
                },
            ],
        ),
        (
            "GPUGlobalTransforms",
            &[ObjectProperty {
                name: "transforms",
                type_name: "Box < [GPULocalTransform] >",
                function: None,
            }],
        ),
        (
            "BuiltMesh",
            &[ObjectProperty {
                name: "id",
                type_name: "MeshId",
                function: None,
            }],
        ),
        (
            "Renderable2D",
            &[
                ObjectProperty {
                    name: "sdf_flags",
                    type_name: "ShapeFlag",
                    function: None,
                },
                ObjectProperty {
                    name: "color",
                    type_name: "Color",
                    function: None,
                },
                ObjectProperty {
                    name: "bounds",
                    type_name: "Bounds",
                    function: None,
                },
                ObjectProperty {
                    name: "transform",
                    type_name: "TransformId",
                    function: None,
                },
                ObjectProperty {
                    name: "image",
                    type_name: "Option < TextureId >",
                    function: None,
                },
                ObjectProperty {
                    name: "border",
                    type_name: "Border",
                    function: None,
                },
            ],
        ),
        (
            "Circle",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "options",
                        type_name: "CircleOptions",
                    }],
                    return_type: Some("Object < Shape >"),
                }),
            }],
        ),
        (
            "Border",
            &[ObjectProperty {
                name: "",
                type_name: "f32",
                function: None,
            }],
        ),
        (
            "Window",
            &[
                ObjectProperty {
                    name: "set_min_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_min_size",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "Option < (u32 , u32) >",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_close_on_esc",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_close_on_esc",
                        parameters: &[FunctionParameter {
                            name: "close_on_esc",
                            type_name: "bool",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_decorations",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_decorations",
                        parameters: &[FunctionParameter {
                            name: "decorations",
                            type_name: "bool",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_resizable",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_resizable",
                        parameters: &[FunctionParameter {
                            name: "resizable",
                            type_name: "bool",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "on",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "on",
                        parameters: &[
                            FunctionParameter {
                                name: "event_name",
                                type_name: "& str",
                            },
                            FunctionParameter {
                                name: "callback",
                                type_name: "impl CallbackFn < Event > + 'static",
                            },
                        ],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "create",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "create",
                        parameters: &[],
                        return_type: Some("Result < Self , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "set_title",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_title",
                        parameters: &[FunctionParameter {
                            name: "title",
                            type_name: "& str",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_auto_resize",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_auto_resize",
                        parameters: &[FunctionParameter {
                            name: "auto_resize",
                            type_name: "bool",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_visible",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_visible",
                        parameters: &[FunctionParameter {
                            name: "visible",
                            type_name: "bool",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "call",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "call",
                        parameters: &[
                            FunctionParameter {
                                name: "event_name",
                                type_name: "& str",
                            },
                            FunctionParameter {
                                name: "event",
                                type_name: "Event",
                            },
                        ],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "get_hovered_file",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "get_hovered_file",
                        parameters: &[FunctionParameter {
                            name: "index",
                            type_name: "u128",
                        }],
                        return_type: Some("Option < String >"),
                    }),
                },
                ObjectProperty {
                    name: "set_fullscreen",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_fullscreen",
                        parameters: &[FunctionParameter {
                            name: "fullscreen",
                            type_name: "bool",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "run",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "run",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "set_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_size",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "(u32 , u32)",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "redraw",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "redraw",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "set_framerate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_framerate",
                        parameters: &[FunctionParameter {
                            name: "framerate",
                            type_name: "Option < u32 >",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[FunctionParameter {
                            name: "options",
                            type_name: "WindowOptions",
                        }],
                        return_type: Some("Result < Self , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "get_dropped_file",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "get_dropped_file",
                        parameters: &[FunctionParameter {
                            name: "index",
                            type_name: "u128",
                        }],
                        return_type: Some("Option < PathBuf >"),
                    }),
                },
                ObjectProperty {
                    name: "set_max_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_max_size",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "Option < (u32 , u32) >",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
            ],
        ),
        (
            "Normal",
            &[ObjectProperty {
                name: "",
                type_name: "[f32 ; 3]",
                function: None,
            }],
        ),
        (
            "TextureTarget",
            &[
                ObjectProperty {
                    name: "texture",
                    type_name: "Texture",
                    function: None,
                },
                ObjectProperty {
                    name: "buffer",
                    type_name: "Option < TextureBuffer >",
                    function: None,
                },
                ObjectProperty {
                    name: "get_rendered_frame_bytes",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "get_rendered_frame_bytes",
                        parameters: &[FunctionParameter {
                            name: "renderer",
                            type_name: "& Renderer",
                        }],
                        return_type: Some("Result < Vec < u8 > , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "from_texture",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
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
                    }),
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
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
                    }),
                },
            ],
        ),
        (
            "Frame",
            &[
                ObjectProperty {
                    name: "present",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "present",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "view",
                    type_name: "wgpu :: TextureView",
                    function: None,
                },
                ObjectProperty {
                    name: "should_present",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "should_present",
                        parameters: &[],
                        return_type: Some("bool"),
                    }),
                },
            ],
        ),
        (
            "WindowOptions",
            &[
                ObjectProperty {
                    name: "min_size",
                    type_name: "Option < (u32 , u32) >",
                    function: None,
                },
                ObjectProperty {
                    name: "auto_resize",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "close_on_esc",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "framerate",
                    type_name: "Option < u32 >",
                    function: None,
                },
                ObjectProperty {
                    name: "size",
                    type_name: "(u32 , u32)",
                    function: None,
                },
                ObjectProperty {
                    name: "resizable",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "max_size",
                    type_name: "Option < (u32 , u32) >",
                    function: None,
                },
                ObjectProperty {
                    name: "decorations",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "fullscreen",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "title",
                    type_name: "String",
                    function: None,
                },
            ],
        ),
        (
            "WindowTarget",
            &[
                ObjectProperty {
                    name: "id",
                    type_name: "WindowId",
                    function: None,
                },
                ObjectProperty {
                    name: "config",
                    type_name: "wgpu :: SurfaceConfiguration",
                    function: None,
                },
                ObjectProperty {
                    name: "scaling_factor",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "surface",
                    type_name: "wgpu :: Surface",
                    function: None,
                },
            ],
        ),
        ("F", &[]),
        (
            "Square",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "size",
                        type_name: "u32",
                    }],
                    return_type: Some("Object < Shape >"),
                }),
            }],
        ),
        (
            "Scene",
            &[
                ObjectProperty {
                    name: "target",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "target",
                        parameters: &[FunctionParameter {
                            name: "descriptor",
                            type_name: "& D",
                        }],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "read_state",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "read_state",
                        parameters: &[],
                        return_type: Some("RwLockReadGuard < '_ , SceneState >"),
                    }),
                },
                ObjectProperty {
                    name: "write_state",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "write_state",
                        parameters: &[],
                        return_type: Some("RwLockWriteGuard < '_ , SceneState >"),
                    }),
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "render",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "render",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "new_unregistered",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new_unregistered",
                        parameters: &[],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "count",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "count",
                        parameters: &[],
                        return_type: Some("u32"),
                    }),
                },
                ObjectProperty {
                    name: "target_with_camera",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "target_with_camera",
                        parameters: &[
                            FunctionParameter {
                                name: "descriptor",
                                type_name: "& D",
                            },
                            FunctionParameter {
                                name: "camera",
                                type_name: "& Object < Camera >",
                            },
                        ],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "add",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "add",
                        parameters: &[FunctionParameter {
                            name: "object",
                            type_name: "& mut impl SceneObject",
                        }],
                        return_type: Some("ObjectId"),
                    }),
                },
                ObjectProperty {
                    name: "print",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "print",
                        parameters: &[],
                        return_type: None,
                    }),
                },
            ],
        ),
        (
            "RenderTargetDescription",
            &[
                ObjectProperty {
                    name: "try_set_camera",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "try_set_camera",
                        parameters: &[FunctionParameter {
                            name: "camera",
                            type_name: "& Object < Camera >",
                        }],
                        return_type: Some("Result < & mut Self , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "after_render",
                    type_name: "Option < Callback < Vec < u8 > > >",
                    function: None,
                },
                ObjectProperty {
                    name: "set_clear_color",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_clear_color",
                        parameters: &[FunctionParameter {
                            name: "clear_color",
                            type_name: "components :: Color",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "after_render",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "after_render",
                        parameters: &[FunctionParameter {
                            name: "callback",
                            type_name: "impl CallbackFn < Vec < u8 > > + 'static",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "target_id",
                    type_name: "TargetId",
                    function: None,
                },
                ObjectProperty {
                    name: "camera_id",
                    type_name: "Option < ObjectId >",
                    function: None,
                },
                ObjectProperty {
                    name: "from_window_id",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_window_id",
                        parameters: &[
                            FunctionParameter {
                                name: "window_id",
                                type_name: "WindowId",
                            },
                            FunctionParameter {
                                name: "size",
                                type_name: "Quad",
                            },
                        ],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "from_texture",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_texture",
                        parameters: &[FunctionParameter {
                            name: "texture",
                            type_name: "& Texture",
                        }],
                        return_type: Some("Result < Self , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "set_camera_id",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_camera_id",
                        parameters: &[FunctionParameter {
                            name: "camera_id",
                            type_name: "ObjectId",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "clear_color",
                    type_name: "components :: Color",
                    function: None,
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[
                            FunctionParameter {
                                name: "target_id",
                                type_name: "TargetId",
                            },
                            FunctionParameter {
                                name: "target_size",
                                type_name: "Quad",
                            },
                        ],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "create_texture_target",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "create_texture_target",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "Quad",
                        }],
                        return_type: Some("Result < Self , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "before_render",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "before_render",
                        parameters: &[FunctionParameter {
                            name: "callback",
                            type_name: "impl CallbackFn < () > + 'static",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "before_render",
                    type_name: "Option < Callback < () > >",
                    function: None,
                },
                ObjectProperty {
                    name: "target_size",
                    type_name: "Quad",
                    function: None,
                },
            ],
        ),
        (
            "App",
            &[
                ObjectProperty {
                    name: "scenes",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "scenes",
                        parameters: &[],
                        return_type: Some("Arc < RwLock < Scenes > >"),
                    }),
                },
                ObjectProperty {
                    name: "windows",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "windows",
                        parameters: &[],
                        return_type: Some("Arc < RwLock < Windows > >"),
                    }),
                },
                ObjectProperty {
                    name: "dispatch_event",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "dispatch_event",
                        parameters: &[FunctionParameter {
                            name: "event",
                            type_name: "Event",
                        }],
                        return_type: Some("Result < () , Error >"),
                    }),
                },
            ],
        ),
        (
            "AppState",
            &[
                ObjectProperty {
                    name: "options",
                    type_name: "AppOptions",
                    function: None,
                },
                ObjectProperty {
                    name: "windows",
                    type_name: "Arc < RwLock < Windows > >",
                    function: None,
                },
                ObjectProperty {
                    name: "scenes",
                    type_name: "Arc < RwLock < Scenes > >",
                    function: None,
                },
            ],
        ),
        (
            "Windows",
            &[ObjectProperty {
                name: "keys",
                type_name: "Vec < WindowId >",
                function: None,
            }],
        ),
        (
            "Camera",
            &[
                ObjectProperty {
                    name: "projection",
                    type_name: "Projection",
                    function: None,
                },
                ObjectProperty {
                    name: "z_near",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "z_far",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "set_projection",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_projection",
                        parameters: &[FunctionParameter {
                            name: "projection",
                            type_name: "Projection",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_near_plane",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_near_plane",
                        parameters: &[FunctionParameter {
                            name: "z_near",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[FunctionParameter {
                            name: "options",
                            type_name: "CameraOptions",
                        }],
                        return_type: Some("Object < Self >"),
                    }),
                },
                ObjectProperty {
                    name: "perspective",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "perspective",
                        parameters: &[FunctionParameter {
                            name: "fov_y",
                            type_name: "f32",
                        }],
                        return_type: Some("Object < Self >"),
                    }),
                },
                ObjectProperty {
                    name: "set_far_plane",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_far_plane",
                        parameters: &[FunctionParameter {
                            name: "z_far",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "from_target_size",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_target_size",
                        parameters: &[FunctionParameter {
                            name: "quad",
                            type_name: "Quad",
                        }],
                        return_type: Some("Object < Self >"),
                    }),
                },
                ObjectProperty {
                    name: "orthographic",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "orthographic",
                        parameters: &[
                            FunctionParameter {
                                name: "center",
                                type_name: "V",
                            },
                            FunctionParameter {
                                name: "size",
                                type_name: "Quad",
                            },
                        ],
                        return_type: Some("Object < Self >"),
                    }),
                },
            ],
        ),
        (
            "ShapeOptions",
            &[
                ObjectProperty {
                    name: "bounds",
                    type_name: "Quad",
                    function: None,
                },
                ObjectProperty {
                    name: "border",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "color",
                    type_name: "Color",
                    function: None,
                },
            ],
        ),
        (
            "VertexIds",
            &[
                ObjectProperty {
                    name: "offset",
                    type_name: "wgpu :: BufferAddress",
                    function: None,
                },
                ObjectProperty {
                    name: "format",
                    type_name: "wgpu :: IndexFormat",
                    function: None,
                },
                ObjectProperty {
                    name: "count",
                    type_name: "u32",
                    function: None,
                },
            ],
        ),
        (
            "Plane",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "size",
                        type_name: "f32",
                    }],
                    return_type: Some("Object < Mesh >"),
                }),
            }],
        ),
        (
            "QuadVertexExtra",
            &[
                ObjectProperty {
                    name: "color",
                    type_name: "Color",
                    function: None,
                },
                ObjectProperty {
                    name: "z",
                    type_name: "f32",
                    function: None,
                },
            ],
        ),
        (
            "SceneState",
            &[
                ObjectProperty {
                    name: "id",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "id",
                        parameters: &[],
                        return_type: Some("SceneId"),
                    }),
                },
                ObjectProperty {
                    name: "world",
                    type_name: "hecs :: World",
                    function: None,
                },
            ],
        ),
        (
            "ShapeFlag",
            &[ObjectProperty {
                name: "",
                type_name: "f32",
                function: None,
            }],
        ),
        (
            "ShapeBuilder",
            &[
                ObjectProperty {
                    name: "stroke",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "stroke",
                        parameters: &[
                            FunctionParameter {
                                name: "path",
                                type_name: "& lyon :: path :: Path",
                            },
                            FunctionParameter {
                                name: "options",
                                type_name: "& StrokeOptions",
                            },
                        ],
                        return_type: Some("Primitive"),
                    }),
                },
                ObjectProperty {
                    name: "fill",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "fill",
                        parameters: &[FunctionParameter {
                            name: "path",
                            type_name: "& lyon :: path :: Path",
                        }],
                        return_type: Some("Primitive"),
                    }),
                },
            ],
        ),
        (
            "GPULocalTransform",
            &[
                ObjectProperty {
                    name: "position",
                    type_name: "[f32 ; 4]",
                    function: None,
                },
                ObjectProperty {
                    name: "inverse_matrix",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "inverse_matrix",
                        parameters: &[],
                        return_type: Some("Mat4"),
                    }),
                },
                ObjectProperty {
                    name: "to_local_transform",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "to_local_transform",
                        parameters: &[],
                        return_type: Some("LocalTransform"),
                    }),
                },
                ObjectProperty {
                    name: "scale",
                    type_name: "[f32 ; 4]",
                    function: None,
                },
                ObjectProperty {
                    name: "rotation",
                    type_name: "[f32 ; 4]",
                    function: None,
                },
            ],
        ),
        (
            "MeshData",
            &[
                ObjectProperty {
                    name: "bound_radius",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "vertex_count",
                    type_name: "u32",
                    function: None,
                },
                ObjectProperty {
                    name: "vertex_data",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "vertex_data",
                        parameters: &[],
                        return_type: Some("Option < & VertexData >"),
                    }),
                },
                ObjectProperty {
                    name: "buffer",
                    type_name: "wgpu :: Buffer",
                    function: None,
                },
                ObjectProperty {
                    name: "vertex_slice",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "vertex_slice",
                        parameters: &[],
                        return_type: Some("wgpu :: BufferSlice"),
                    }),
                },
                ObjectProperty {
                    name: "vertex_ids",
                    type_name: "Option < VertexIds >",
                    function: None,
                },
            ],
        ),
        (
            "Texture",
            &[
                ObjectProperty {
                    name: "from_file",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_file",
                        parameters: &[FunctionParameter {
                            name: "path",
                            type_name: "impl AsRef < Path >",
                        }],
                        return_type: Some("Result < (TextureId , Quad) , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "size",
                    type_name: "wgpu :: Extent3d",
                    function: None,
                },
                ObjectProperty {
                    name: "view",
                    type_name: "wgpu :: TextureView",
                    function: None,
                },
                ObjectProperty {
                    name: "data",
                    type_name: "wgpu :: Texture",
                    function: None,
                },
                ObjectProperty {
                    name: "id",
                    type_name: "TextureId",
                    function: None,
                },
                ObjectProperty {
                    name: "format",
                    type_name: "wgpu :: TextureFormat",
                    function: None,
                },
                ObjectProperty {
                    name: "sampler",
                    type_name: "wgpu :: Sampler",
                    function: None,
                },
                ObjectProperty {
                    name: "from_bytes",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_bytes",
                        parameters: &[FunctionParameter {
                            name: "bytes",
                            type_name: "& [u8]",
                        }],
                        return_type: Some("Result < (TextureId , Quad) , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "create_depth_texture",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "create_depth_texture",
                        parameters: &[FunctionParameter {
                            name: "size",
                            type_name: "wgpu :: Extent3d",
                        }],
                        return_type: Some("Result < (TextureId , Quad) , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "create_blank_pixel",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "create_blank_pixel",
                        parameters: &[],
                        return_type: Some("Result < (TextureId , Quad) , Error >"),
                    }),
                },
            ],
        ),
        (
            "Sprite",
            &[
                ObjectProperty {
                    name: "image",
                    type_name: "TextureId",
                    function: None,
                },
                ObjectProperty {
                    name: "load_image",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "load_image",
                        parameters: &[FunctionParameter {
                            name: "path",
                            type_name: "impl AsRef < Path >",
                        }],
                        return_type: Some("(TextureId , Quad)"),
                    }),
                },
                ObjectProperty {
                    name: "clip_region",
                    type_name: "Option < Quad >",
                    function: None,
                },
                ObjectProperty {
                    name: "image_size",
                    type_name: "Quad",
                    function: None,
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[FunctionParameter {
                            name: "image_path",
                            type_name: "impl AsRef < Path >",
                        }],
                        return_type: Some("Object < Sprite >"),
                    }),
                },
            ],
        ),
        (
            "LightOptions",
            &[
                ObjectProperty {
                    name: "variant",
                    type_name: "LightType",
                    function: None,
                },
                ObjectProperty {
                    name: "color",
                    type_name: "Color",
                    function: None,
                },
                ObjectProperty {
                    name: "intensity",
                    type_name: "f32",
                    function: None,
                },
            ],
        ),
        ("TextureId", &[]),
        ("MeshId", &[]),
        (
            "Empty",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[],
                    return_type: Some("Object < Self >"),
                }),
            }],
        ),
        (
            "MeshBuilder",
            &[
                ObjectProperty {
                    name: "vertex",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "vertex",
                        parameters: &[FunctionParameter {
                            name: "data",
                            type_name: "& [T]",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "index",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "index",
                        parameters: &[FunctionParameter {
                            name: "data",
                            type_name: "& [u16]",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "build",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "build",
                        parameters: &[],
                        return_type: Some("Result < BuiltMesh , Error >"),
                    }),
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "name",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "name",
                        parameters: &[FunctionParameter {
                            name: "name",
                            type_name: "& str",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "radius",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "radius",
                        parameters: &[FunctionParameter {
                            name: "radius",
                            type_name: "f32",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
            ],
        ),
        (
            "Point",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[],
                    return_type: Some("Object < Shape >"),
                }),
            }],
        ),
        (
            "Box",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "dimensions",
                        type_name: "V",
                    }],
                    return_type: Some("Object < Mesh >"),
                }),
            }],
        ),
        (
            "FragmentColor",
            &[
                ObjectProperty {
                    name: "app",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "app",
                        parameters: &[],
                        return_type: Some("Arc < RwLock < App > >"),
                    }),
                },
                ObjectProperty {
                    name: "run",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "run",
                        parameters: &[],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "config",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "config",
                        parameters: &[FunctionParameter {
                            name: "options",
                            type_name: "AppOptions",
                        }],
                        return_type: None,
                    }),
                },
            ],
        ),
        (
            "Cube",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "size",
                        type_name: "f32",
                    }],
                    return_type: Some("Object < Mesh >"),
                }),
            }],
        ),
        (
            "Controller",
            &[
                ObjectProperty {
                    name: "speed",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "is_right_pressed",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[FunctionParameter {
                            name: "speed",
                            type_name: "f32",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "handle_event",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "handle_event",
                        parameters: &[FunctionParameter {
                            name: "event",
                            type_name: "& WindowEvent",
                        }],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "is_backward_pressed",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "is_forward_pressed",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "is_left_pressed",
                    type_name: "bool",
                    function: None,
                },
                ObjectProperty {
                    name: "update_transform",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "update_transform",
                        parameters: &[FunctionParameter {
                            name: "_transform",
                            type_name: "& mut Transform",
                        }],
                        return_type: None,
                    }),
                },
            ],
        ),
        (
            "CameraOptions",
            &[
                ObjectProperty {
                    name: "z_near",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "z_far",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "projection",
                    type_name: "Projection",
                    function: None,
                },
            ],
        ),
        (
            "AppOptions",
            &[
                ObjectProperty {
                    name: "renderer",
                    type_name: "RendererOptions",
                    function: None,
                },
                ObjectProperty {
                    name: "log_level",
                    type_name: "String",
                    function: None,
                },
            ],
        ),
        (
            "Bounds",
            &[ObjectProperty {
                name: "",
                type_name: "Quad",
                function: None,
            }],
        ),
        (
            "LocalTransform",
            &[
                ObjectProperty {
                    name: "position",
                    type_name: "glam :: Vec3",
                    function: None,
                },
                ObjectProperty {
                    name: "rotation",
                    type_name: "glam :: Quat",
                    function: None,
                },
                ObjectProperty {
                    name: "scale",
                    type_name: "glam :: Vec3",
                    function: None,
                },
            ],
        ),
        (
            "Radius",
            &[ObjectProperty {
                name: "",
                type_name: "f32",
                function: None,
            }],
        ),
        (
            "TransformId",
            &[
                ObjectProperty {
                    name: "root",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "root",
                        parameters: &[],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "",
                    type_name: "u32",
                    function: None,
                },
            ],
        ),
        (
            "Scenes",
            &[ObjectProperty {
                name: "keys",
                type_name: "Vec < SceneId >",
                function: None,
            }],
        ),
        (
            "Transform",
            &[
                ObjectProperty {
                    name: "set_rotation_radians",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_rotation_radians",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "radians",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_parent",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_parent",
                        parameters: &[FunctionParameter {
                            name: "parent",
                            type_name: "TransformId",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_rotation_degrees",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_rotation_degrees",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_rotation_quaternion",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_rotation_quaternion",
                        parameters: &[FunctionParameter {
                            name: "quat",
                            type_name: "Q",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "root",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "root",
                        parameters: &[],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotate",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "pre_rotate_degrees",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pre_rotate_degrees",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "has_moved",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "has_moved",
                        parameters: &[],
                        return_type: Some("bool"),
                    }),
                },
                ObjectProperty {
                    name: "local_transform",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "local_transform",
                        parameters: &[],
                        return_type: Some("LocalTransform"),
                    }),
                },
                ObjectProperty {
                    name: "pre_translate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pre_translate",
                        parameters: &[FunctionParameter {
                            name: "offset",
                            type_name: "Vec3",
                        }],
                        return_type: None,
                    }),
                },
                ObjectProperty {
                    name: "pre_rotate_radians",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pre_rotate_radians",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "radians",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "translate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "translate",
                        parameters: &[FunctionParameter {
                            name: "offset",
                            type_name: "Vec3",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_position",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_position",
                        parameters: &[FunctionParameter {
                            name: "position",
                            type_name: "Vec3",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotate_degrees",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotate_degrees",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "set_scale",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_scale",
                        parameters: &[FunctionParameter {
                            name: "scale",
                            type_name: "Vec3",
                        }],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotation",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotation",
                        parameters: &[],
                        return_type: Some("(Vec3 , f32)"),
                    }),
                },
                ObjectProperty {
                    name: "set_rotation",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "set_rotation",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotation_quaternion",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotation_quaternion",
                        parameters: &[],
                        return_type: Some("Quaternion"),
                    }),
                },
                ObjectProperty {
                    name: "position",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "position",
                        parameters: &[],
                        return_type: Some("Vec3"),
                    }),
                },
                ObjectProperty {
                    name: "rotation_radians",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotation_radians",
                        parameters: &[],
                        return_type: Some("(Vec3 , f32)"),
                    }),
                },
                ObjectProperty {
                    name: "parent",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "parent",
                        parameters: &[],
                        return_type: Some("TransformId"),
                    }),
                },
                ObjectProperty {
                    name: "rotate_radians",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotate_radians",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "radians",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "rotation_degrees",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "rotation_degrees",
                        parameters: &[],
                        return_type: Some("(Vec3 , f32)"),
                    }),
                },
                ObjectProperty {
                    name: "pre_rotate",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "pre_rotate",
                        parameters: &[
                            FunctionParameter {
                                name: "axis",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "degrees",
                                type_name: "f32",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "look_at",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "look_at",
                        parameters: &[
                            FunctionParameter {
                                name: "target",
                                type_name: "Vec3",
                            },
                            FunctionParameter {
                                name: "up",
                                type_name: "Vec3",
                            },
                        ],
                        return_type: Some("& mut Self"),
                    }),
                },
                ObjectProperty {
                    name: "scale",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "scale",
                        parameters: &[],
                        return_type: Some("glam :: Vec3"),
                    }),
                },
            ],
        ),
        (
            "QuadVertex",
            &[
                ObjectProperty {
                    name: "bounds",
                    type_name: "Quad",
                    function: None,
                },
                ObjectProperty {
                    name: "extra",
                    type_name: "& 'x X",
                    function: None,
                },
                ObjectProperty {
                    name: "tex_coords",
                    type_name: "Quad",
                    function: None,
                },
                ObjectProperty {
                    name: "pixel_coords",
                    type_name: "Quad",
                    function: None,
                },
            ],
        ),
        (
            "Color",
            &[
                ObjectProperty {
                    name: "from_rgb_alpha",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
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
                    }),
                },
                ObjectProperty {
                    name: "from_css",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_css",
                        parameters: &[FunctionParameter {
                            name: "color",
                            type_name: "& str",
                        }],
                        return_type: Some("Result < Self , csscolorparser :: ParseColorError >"),
                    }),
                },
                ObjectProperty {
                    name: "blue",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "blue",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "alpha",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "alpha",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "red",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "red",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
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
                    }),
                },
                ObjectProperty {
                    name: "from_hex",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_hex",
                        parameters: &[FunctionParameter {
                            name: "hex",
                            type_name: "& str",
                        }],
                        return_type: Some("Result < Self , csscolorparser :: ParseColorError >"),
                    }),
                },
                ObjectProperty {
                    name: "",
                    type_name: "u32",
                    function: None,
                },
                ObjectProperty {
                    name: "green",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "green",
                        parameters: &[],
                        return_type: Some("f32"),
                    }),
                },
                ObjectProperty {
                    name: "to_array",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "to_array",
                        parameters: &[],
                        return_type: Some("[f32 ; 4]"),
                    }),
                },
                ObjectProperty {
                    name: "from_rgba",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "from_rgba",
                        parameters: &[FunctionParameter {
                            name: "d",
                            type_name: "[f32 ; 4]",
                        }],
                        return_type: Some("Self"),
                    }),
                },
                ObjectProperty {
                    name: "into_vec4_gamma",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "into_vec4_gamma",
                        parameters: &[],
                        return_type: Some("[f32 ; 4]"),
                    }),
                },
            ],
        ),
        (
            "CircleOptions",
            &[
                ObjectProperty {
                    name: "color",
                    type_name: "Color",
                    function: None,
                },
                ObjectProperty {
                    name: "border",
                    type_name: "f32",
                    function: None,
                },
                ObjectProperty {
                    name: "radius",
                    type_name: "f32",
                    function: None,
                },
            ],
        ),
        ("IsHidden", &[]),
        (
            "Position",
            &[ObjectProperty {
                name: "",
                type_name: "[f32 ; 3]",
                function: None,
            }],
        ),
        (
            "Rectangle",
            &[ObjectProperty {
                name: "new",
                type_name: "FunctionSignature",
                function: Some(FunctionSignature {
                    name: "new",
                    parameters: &[FunctionParameter {
                        name: "options",
                        type_name: "ShapeOptions",
                    }],
                    return_type: Some("Object < Shape >"),
                }),
            }],
        ),
        ("Vertex", &[]),
        ("ObjectBuilder", &[]),
        (
            "Shape",
            &[
                ObjectProperty {
                    name: "new",
                    type_name: "FunctionSignature",
                    function: Some(FunctionSignature {
                        name: "new",
                        parameters: &[
                            FunctionParameter {
                                name: "options",
                                type_name: "& ShapeOptions",
                            },
                            FunctionParameter {
                                name: "shape_type",
                                type_name: "ShapeType",
                            },
                        ],
                        return_type: Some("Object < Self >"),
                    }),
                },
                ObjectProperty {
                    name: "transform_id",
                    type_name: "TransformId",
                    function: None,
                },
            ],
        ),
        (
            "TextureCoordinates",
            &[ObjectProperty {
                name: "",
                type_name: "[u16 ; 2]",
                function: None,
            }],
        ),
    ],
};
