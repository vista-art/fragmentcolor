
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
    key: 7485420634051515786,
    disps: &[
        (0, 16),
        (7, 13),
        (1, 0),
        (11, 7),
    ],
    entries: &[
        ("SolidConfig", &[]),
        ("Position", &[]),
        ("SpriteMap", &[FunctionSignature { name: "at", parameters: &[FunctionParameter { name: "index", type_name: "mint :: Point2 < usize >" }], return_type: Some("crate :: UvRange") }, ]),
        ("Phong", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "config", type_name: "& PhongConfig" }, FunctionParameter { name: "context", type_name: "& crate :: Context" }], return_type: Some("Self") }, FunctionSignature { name: "new_offscreen", parameters: &[FunctionParameter { name: "config", type_name: "& PhongConfig" }, FunctionParameter { name: "target_info", type_name: "crate :: TargetInfo" }, FunctionParameter { name: "context", type_name: "& crate :: Context" }], return_type: Some("Self") }, ]),
        ("TexCoords", &[]),
        ("WindowBuilder", &[FunctionSignature { name: "title", parameters: &[FunctionParameter { name: "title", type_name: "& str" }], return_type: Some("Self") }, FunctionSignature { name: "size", parameters: &[FunctionParameter { name: "width", type_name: "u32" }, FunctionParameter { name: "height", type_name: "u32" }], return_type: Some("Self") }, FunctionSignature { name: "build", parameters: &[], return_type: Some("Window") }, ]),
        ("Material", &[]),
        ("PhongConfig", &[]),
        ("Flat", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "context", type_name: "& crate :: Context" }], return_type: Some("Self") }, FunctionSignature { name: "new_offscreen", parameters: &[FunctionParameter { name: "target_info", type_name: "crate :: TargetInfo" }, FunctionParameter { name: "context", type_name: "& crate :: Context" }], return_type: Some("Self") }, ]),
        ("Solid", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "config", type_name: "& SolidConfig" }, FunctionParameter { name: "context", type_name: "& crate :: Context" }], return_type: Some("Self") }, FunctionSignature { name: "new_offscreen", parameters: &[FunctionParameter { name: "config", type_name: "& SolidConfig" }, FunctionParameter { name: "target_info", type_name: "crate :: TargetInfo" }, FunctionParameter { name: "context", type_name: "& crate :: Context" }], return_type: Some("Self") }, ]),
        ("asset_gltf.rs", &[FunctionSignature { name: "load_gltf", parameters: &[FunctionParameter { name: "path", type_name: "impl AsRef < Path >" }, FunctionParameter { name: "scene", type_name: "& mut crate :: Scene" }, FunctionParameter { name: "global_parent", type_name: "crate :: NodeRef" }, FunctionParameter { name: "context", type_name: "& mut crate :: Context" }], return_type: Some("Module") }, ]),
        ("RealConfig", &[]),
        ("Ambient", &[]),
        ("Geometry", &[FunctionSignature { name: "cuboid", parameters: &[FunctionParameter { name: "streams", type_name: "super :: Streams" }, FunctionParameter { name: "half_extent", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("Self") }, FunctionSignature { name: "plane", parameters: &[FunctionParameter { name: "size", type_name: "f32" }], return_type: Some("Self") }, FunctionSignature { name: "fill", parameters: &[FunctionParameter { name: "path", type_name: "& Path" }], return_type: Some("Self") }, FunctionSignature { name: "stroke", parameters: &[FunctionParameter { name: "path", type_name: "& Path" }, FunctionParameter { name: "options", type_name: "& StrokeOptions" }], return_type: Some("Self") }, FunctionSignature { name: "sphere", parameters: &[FunctionParameter { name: "streams", type_name: "super :: Streams" }, FunctionParameter { name: "radius", type_name: "f32" }, FunctionParameter { name: "detail", type_name: "usize" }], return_type: Some("Self") }, FunctionSignature { name: "bake", parameters: &[FunctionParameter { name: "context", type_name: "& mut plr :: Context" }], return_type: Some("plr :: Prototype") }, ]),
        ("BufferPool", &[]),
        ("asset_obj.rs", &[FunctionSignature { name: "load_obj", parameters: &[FunctionParameter { name: "path", type_name: "impl AsRef < Path >" }, FunctionParameter { name: "scene", type_name: "& mut crate :: Scene" }, FunctionParameter { name: "node", type_name: "crate :: NodeRef" }, FunctionParameter { name: "context", type_name: "& mut crate :: Context" }], return_type: Some("fxhash :: FxHashMap < String , (crate :: EntityRef , crate :: Prototype) >") }, ]),
        ("Normal", &[]),
        ("Window", &[FunctionSignature { name: "new", parameters: &[], return_type: Some("WindowBuilder") }, FunctionSignature { name: "run", parameters: &[FunctionParameter { name: "mut runner", type_name: "impl 'static + FnMut (Event)" }], return_type: Some("!") }, ]),
        ("Real", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "config", type_name: "& RealConfig" }, FunctionParameter { name: "context", type_name: "& crate :: Context" }], return_type: Some("Self") }, FunctionSignature { name: "new_offscreen", parameters: &[FunctionParameter { name: "config", type_name: "& RealConfig" }, FunctionParameter { name: "target_info", type_name: "crate :: TargetInfo" }, FunctionParameter { name: "context", type_name: "& crate :: Context" }], return_type: Some("Self") }, ]),
    ],
};
