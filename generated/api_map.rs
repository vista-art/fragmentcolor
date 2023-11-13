
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
        (0, 1),
        (0, 8),
        (0, 32),
        (2, 14),
        (0, 17),
        (0, 33),
        (0, 38),
        (8, 7),
        (0, 0),
        (40, 7),
        (0, 25),
        (2, 12),
        (5, 42),
    ],
    entries: &[
        ("Sprite", &[FunctionSignature { name: "uv", parameters: &[FunctionParameter { name: "uv", type_name: "UvRange" }], return_type: Some("& mut Self") }, FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "image", type_name: "TextureId" }, FunctionParameter { name: "uv", type_name: "UvRange" }], return_type: Some("Self") }, ]),
        ("WindowOptions", &[]),
        ("Light", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "options", type_name: "LightOptions" }], return_type: Some("Self") }, FunctionSignature { name: "set_color", parameters: &[FunctionParameter { name: "color", type_name: "Color" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_intensity", parameters: &[FunctionParameter { name: "intensity", type_name: "f32" }], return_type: Some("& mut Self") }, ]),
        ("Material", &[]),
        ("SamplerOptions", &[]),
        ("Target", &[]),
        ("Renderer", &[FunctionSignature { name: "render_pass_flat", parameters: &[FunctionParameter { name: "scene", type_name: "& Scene" }], return_type: Some("Result < () , wgpu :: SurfaceError >") }, FunctionSignature { name: "render_pass_solid", parameters: &[FunctionParameter { name: "scene", type_name: "& Scene" }], return_type: Some("Result < () , wgpu :: SurfaceError >") }, FunctionSignature { name: "load_image", parameters: &[FunctionParameter { name: "path_ref", type_name: "impl AsRef < Path >" }], return_type: Some("Result < TextureId , Error >") }, FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "options", type_name: "RenderOptions < 'w , W >" }], return_type: Some("Result < Renderer , Error >") }, FunctionSignature { name: "add_texture", parameters: &[FunctionParameter { name: "texture", type_name: "wgpu :: Texture" }], return_type: Some("TextureId") }, FunctionSignature { name: "add_texture_from_bytes", parameters: &[FunctionParameter { name: "desc", type_name: "& wgpu :: TextureDescriptor" }, FunctionParameter { name: "data", type_name: "& [u8]" }], return_type: Some("TextureId") }, FunctionSignature { name: "add_mesh", parameters: &[FunctionParameter { name: "mesh", type_name: "Mesh" }], return_type: Some("MeshId") }, FunctionSignature { name: "add_target", parameters: &[FunctionParameter { name: "window", type_name: "W" }], return_type: Some("Result < TargetId , Error >") }, FunctionSignature { name: "render", parameters: &[FunctionParameter { name: "scene", type_name: "& Scene" }], return_type: Some("Result < () , wgpu :: SurfaceError >") }, ]),
        ("TextureTarget", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "renderer", type_name: "& Renderer" }, FunctionParameter { name: "size", type_name: "wgpu :: Extent3d" }], return_type: Some("Result < Self , Error >") }, FunctionSignature { name: "from_wgpu_texture", parameters: &[FunctionParameter { name: "renderer", type_name: "& Renderer" }, FunctionParameter { name: "texture", type_name: "wgpu :: Texture" }], return_type: Some("Result < Self , Error >") }, FunctionSignature { name: "from_texture", parameters: &[FunctionParameter { name: "renderer", type_name: "& Renderer" }, FunctionParameter { name: "texture", type_name: "Texture" }], return_type: Some("Result < Self , Error >") }, ]),
        ("App", &[FunctionSignature { name: "dispatch_event", parameters: &[FunctionParameter { name: "event", type_name: "Event" }], return_type: Some("Result < () , EventLoopClosed < Event > >") }, FunctionSignature { name: "add_window", parameters: &[FunctionParameter { name: "window", type_name: "& mut W" }], return_type: None }, FunctionSignature { name: "run", parameters: &[], return_type: None }, FunctionSignature { name: "remove_window", parameters: &[FunctionParameter { name: "window", type_name: "W" }], return_type: None }, FunctionSignature { name: "state", parameters: &[], return_type: Some("MutexGuard < '_ , AppState >") }, FunctionSignature { name: "event_loop", parameters: &[], return_type: Some("MutexGuard < '_ , EventLoop < Event > >") }, FunctionSignature { name: "new_window", parameters: &[], return_type: Some("Result < Window , OsError >") }, ]),
        ("Vec", &[]),
        ("PLRender", &[FunctionSignature { name: "app", parameters: &[], return_type: Some("MutexGuard < 'static , App >") }, FunctionSignature { name: "run", parameters: &[], return_type: None }, FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "options", type_name: "AppOptions" }], return_type: Some("MutexGuard < 'static , App >") }, ]),
        ("Resources", &[FunctionSignature { name: "get_mesh", parameters: &[FunctionParameter { name: "id", type_name: "MeshId" }], return_type: Some("& Mesh") }, FunctionSignature { name: "new", parameters: &[], return_type: Some("Self") }, FunctionSignature { name: "add_mesh", parameters: &[FunctionParameter { name: "mesh", type_name: "Mesh" }], return_type: Some("MeshId") }, FunctionSignature { name: "add_texture", parameters: &[FunctionParameter { name: "texture", type_name: "Texture" }], return_type: Some("TextureId") }, FunctionSignature { name: "get_texture", parameters: &[FunctionParameter { name: "id", type_name: "TextureId" }], return_type: Some("& Texture") }, ]),
        ("Ambient", &[]),
        ("Scenes", &[]),
        ("Real", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "config", type_name: "& RealConfig" }, FunctionParameter { name: "renderer", type_name: "& 'r Renderer" }], return_type: Some("Self") }, ]),
        ("Normal", &[]),
        ("Mesh", &[FunctionSignature { name: "vertex_data", parameters: &[], return_type: Some("Option < & VertexData >") }, FunctionSignature { name: "vertex_slice", parameters: &[], return_type: Some("wgpu :: BufferSlice") }, ]),
        ("Solid", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "config", type_name: "& SolidConfig" }, FunctionParameter { name: "renderer", type_name: "& 'r Renderer" }], return_type: Some("Self") }, ]),
        ("Windows", &[]),
        ("SceneObject", &[FunctionSignature { name: "rotate_radians", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "radians", type_name: "f32" }], return_type: None }, FunctionSignature { name: "local_transform", parameters: &[], return_type: Some("Transform") }, FunctionSignature { name: "set_rotation", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_rotation_quaternion", parameters: &[FunctionParameter { name: "quat", type_name: "mint :: Quaternion < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "pre_rotate", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: None }, FunctionSignature { name: "set_parent", parameters: &[FunctionParameter { name: "parent", type_name: "NodeId" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_intensity", parameters: &[FunctionParameter { name: "intensity", type_name: "f32" }], return_type: Some("& mut Self") }, FunctionSignature { name: "rotation_quaternion", parameters: &[], return_type: Some("mint :: Quaternion < f32 >") }, FunctionSignature { name: "rotation", parameters: &[], return_type: Some("(mint :: Vector3 < f32 > , f32)") }, FunctionSignature { name: "add_component", parameters: &[FunctionParameter { name: "component", type_name: "C" }], return_type: Some("& mut Self") }, FunctionSignature { name: "rotation_radians", parameters: &[], return_type: Some("(mint :: Vector3 < f32 > , f32)") }, FunctionSignature { name: "set_rotation_degrees", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: Some("& mut Self") }, FunctionSignature { name: "position", parameters: &[], return_type: Some("mint :: Vector3 < f32 >") }, FunctionSignature { name: "add_components", parameters: &[FunctionParameter { name: "bundle", type_name: "B" }], return_type: Some("& mut Self") }, FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "scene", type_name: "Arc < RwLock < SceneState > >" }, FunctionParameter { name: "object", type_name: "T" }], return_type: Some("Self") }, FunctionSignature { name: "rotate_degrees", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: None }, FunctionSignature { name: "translate", parameters: &[FunctionParameter { name: "offset", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "look_at", parameters: &[FunctionParameter { name: "target", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "up", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "parent", parameters: &[], return_type: Some("NodeId") }, FunctionSignature { name: "scale", parameters: &[], return_type: Some("glam :: Vec3") }, FunctionSignature { name: "set_position", parameters: &[FunctionParameter { name: "position", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "pre_rotate_degrees", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: None }, FunctionSignature { name: "pre_rotate_radians", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "radians", type_name: "f32" }], return_type: None }, FunctionSignature { name: "set_scale", parameters: &[FunctionParameter { name: "scale", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "has_moved", parameters: &[], return_type: Some("bool") }, FunctionSignature { name: "pre_translate", parameters: &[FunctionParameter { name: "offset", type_name: "mint :: Vector3 < f32 >" }], return_type: None }, FunctionSignature { name: "set_rotation_radians", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "radians", type_name: "f32" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_mesh", parameters: &[FunctionParameter { name: "mesh_id", type_name: "MeshId" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_color", parameters: &[FunctionParameter { name: "color", type_name: "Color" }], return_type: Some("& mut Self") }, FunctionSignature { name: "uv", parameters: &[FunctionParameter { name: "uv", type_name: "UvRange" }], return_type: Some("& mut Self") }, FunctionSignature { name: "rotation_degrees", parameters: &[], return_type: Some("(mint :: Vector3 < f32 > , f32)") }, FunctionSignature { name: "state_mut", parameters: &[], return_type: Some("RwLockWriteGuard < '_ , SceneState >") }, FunctionSignature { name: "rotate", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: None }, FunctionSignature { name: "scene", parameters: &[], return_type: Some("RwLockReadGuard < '_ , SceneState >") }, FunctionSignature { name: "id", parameters: &[], return_type: Some("Option < ObjectId >") }, ]),
        ("Phong", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "config", type_name: "& PhongConfig" }, FunctionParameter { name: "renderer", type_name: "& 'r Renderer" }], return_type: Some("Self") }, ]),
        ("PhongConfig", &[]),
        ("BufferSize", &[FunctionSignature { name: "size", parameters: &[], return_type: Some("u64") }, FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "width", type_name: "usize" }, FunctionParameter { name: "height", type_name: "usize" }], return_type: Some("Self") }, ]),
        ("TextureCoordinates", &[]),
        ("GlobalTransforms", &[]),
        ("TextureRegion", &[FunctionSignature { name: "height", parameters: &[], return_type: Some("u32") }, FunctionSignature { name: "for_region", parameters: &[FunctionParameter { name: "x", type_name: "u32" }, FunctionParameter { name: "y", type_name: "u32" }, FunctionParameter { name: "width", type_name: "u32" }, FunctionParameter { name: "height", type_name: "u32" }], return_type: Some("Self") }, FunctionSignature { name: "clamp_with_intersection", parameters: &[FunctionParameter { name: "self_point", type_name: "(i32 , i32)" }, FunctionParameter { name: "other_point", type_name: "(i32 , i32)" }, FunctionParameter { name: "size", type_name: "(i32 , i32)" }, FunctionParameter { name: "other", type_name: "& mut TextureRegion" }], return_type: None }, FunctionSignature { name: "for_whole_size", parameters: &[FunctionParameter { name: "width", type_name: "u32" }, FunctionParameter { name: "height", type_name: "u32" }], return_type: Some("Self") }, FunctionSignature { name: "clamp", parameters: &[FunctionParameter { name: "width", type_name: "u32" }, FunctionParameter { name: "height", type_name: "u32" }], return_type: None }, FunctionSignature { name: "encompass", parameters: &[FunctionParameter { name: "x", type_name: "u32" }, FunctionParameter { name: "y", type_name: "u32" }], return_type: None }, FunctionSignature { name: "encompassing_pixels", parameters: &[FunctionParameter { name: "a", type_name: "(u32 , u32)" }, FunctionParameter { name: "b", type_name: "(u32 , u32)" }], return_type: Some("Self") }, FunctionSignature { name: "union", parameters: &[FunctionParameter { name: "other", type_name: "TextureRegion" }], return_type: None }, FunctionSignature { name: "for_region_i32", parameters: &[FunctionParameter { name: "x", type_name: "i32" }, FunctionParameter { name: "y", type_name: "i32" }, FunctionParameter { name: "width", type_name: "i32" }, FunctionParameter { name: "height", type_name: "i32" }], return_type: Some("Self") }, FunctionSignature { name: "encompassing_pixels_i32", parameters: &[FunctionParameter { name: "a", type_name: "(i32 , i32)" }, FunctionParameter { name: "b", type_name: "(i32 , i32)" }], return_type: Some("Self") }, FunctionSignature { name: "for_pixel", parameters: &[FunctionParameter { name: "x", type_name: "u32" }, FunctionParameter { name: "y", type_name: "u32" }], return_type: Some("Self") }, FunctionSignature { name: "intersects", parameters: &[FunctionParameter { name: "other", type_name: "TextureRegion" }], return_type: Some("bool") }, FunctionSignature { name: "width", parameters: &[], return_type: Some("u32") }, ]),
        ("F", &[]),
        ("Node", &[FunctionSignature { name: "parent", parameters: &[], return_type: Some("NodeId") }, FunctionSignature { name: "rotation", parameters: &[], return_type: Some("(mint :: Vector3 < f32 > , f32)") }, FunctionSignature { name: "set_scale", parameters: &[FunctionParameter { name: "scale", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_rotation_quaternion", parameters: &[FunctionParameter { name: "quat", type_name: "mint :: Quaternion < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "pre_rotate", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: None }, FunctionSignature { name: "rotate_degrees", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: None }, FunctionSignature { name: "set_parent", parameters: &[FunctionParameter { name: "parent", type_name: "NodeId" }], return_type: Some("& mut Self") }, FunctionSignature { name: "has_moved", parameters: &[], return_type: Some("bool") }, FunctionSignature { name: "rotation_quaternion", parameters: &[], return_type: Some("mint :: Quaternion < f32 >") }, FunctionSignature { name: "set_rotation_degrees", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: Some("& mut Self") }, FunctionSignature { name: "rotation_degrees", parameters: &[], return_type: Some("(mint :: Vector3 < f32 > , f32)") }, FunctionSignature { name: "set_rotation_radians", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "radians", type_name: "f32" }], return_type: Some("& mut Self") }, FunctionSignature { name: "pre_rotate_radians", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "radians", type_name: "f32" }], return_type: None }, FunctionSignature { name: "look_at", parameters: &[FunctionParameter { name: "target", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "up", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "local_transform", parameters: &[], return_type: Some("Transform") }, FunctionSignature { name: "id", parameters: &[], return_type: Some("NodeId") }, FunctionSignature { name: "rotation_radians", parameters: &[], return_type: Some("(mint :: Vector3 < f32 > , f32)") }, FunctionSignature { name: "rotate", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: None }, FunctionSignature { name: "pre_rotate_degrees", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: None }, FunctionSignature { name: "set_position", parameters: &[FunctionParameter { name: "position", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "position", parameters: &[], return_type: Some("mint :: Vector3 < f32 >") }, FunctionSignature { name: "translate", parameters: &[FunctionParameter { name: "offset", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "rotate_radians", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "radians", type_name: "f32" }], return_type: None }, FunctionSignature { name: "set_rotation", parameters: &[FunctionParameter { name: "axis", type_name: "mint :: Vector3 < f32 >" }, FunctionParameter { name: "degrees", type_name: "f32" }], return_type: Some("& mut Self") }, FunctionSignature { name: "scale", parameters: &[], return_type: Some("glam :: Vec3") }, FunctionSignature { name: "pre_translate", parameters: &[FunctionParameter { name: "offset", type_name: "mint :: Vector3 < f32 >" }], return_type: None }, ]),
        ("EventLoop", &[FunctionSignature { name: "window_target", parameters: &[], return_type: Some("& EventLoopWindowTarget < Event >") }, FunctionSignature { name: "run", parameters: &[FunctionParameter { name: "runner", type_name: "EventLoopRunner" }, FunctionParameter { name: "app", type_name: "Arc < Mutex < AppState > >" }], return_type: None }, FunctionSignature { name: "new", parameters: &[], return_type: Some("Self") }, FunctionSignature { name: "create_dispatcher", parameters: &[], return_type: Some("EventLoopProxy < Event >") }, ]),
        ("Targets", &[FunctionSignature { name: "new", parameters: &[], return_type: Some("Self") }, ]),
        ("BufferPool", &[]),
        ("app_event_loop.rs", &[FunctionSignature { name: "run_event_loop", parameters: &[FunctionParameter { name: "event_loop", type_name: "WinitEventLoop < Event >" }, FunctionParameter { name: "app", type_name: "Arc < Mutex < AppState > >" }], return_type: None }, ]),
        ("Geometry", &[FunctionSignature { name: "fill", parameters: &[FunctionParameter { name: "path", type_name: "& Path" }], return_type: Some("Self") }, FunctionSignature { name: "cuboid", parameters: &[FunctionParameter { name: "vertex_types", type_name: "super :: VertexTypes" }, FunctionParameter { name: "half_extent", type_name: "mint :: Vector3 < f32 >" }], return_type: Some("Self") }, FunctionSignature { name: "sphere", parameters: &[FunctionParameter { name: "vertex_types", type_name: "super :: VertexTypes" }, FunctionParameter { name: "radius", type_name: "f32" }, FunctionParameter { name: "detail", type_name: "usize" }], return_type: Some("Self") }, FunctionSignature { name: "stroke", parameters: &[FunctionParameter { name: "path", type_name: "& Path" }, FunctionParameter { name: "options", type_name: "& StrokeOptions" }], return_type: Some("Self") }, FunctionSignature { name: "build_mesh", parameters: &[FunctionParameter { name: "renderer", type_name: "& mut Renderer" }], return_type: Some("mesh :: MeshPrototype") }, FunctionSignature { name: "plane", parameters: &[FunctionParameter { name: "size", type_name: "f32" }], return_type: Some("Self") }, ]),
        ("RenderOptions", &[]),
        ("Animator", &[FunctionSignature { name: "tick", parameters: &[FunctionParameter { name: "scene", type_name: "& mut Scene" }], return_type: None }, FunctionSignature { name: "update_uv", parameters: &[FunctionParameter { name: "scene", type_name: "& mut Scene" }], return_type: None }, FunctionSignature { name: "switch", parameters: &[FunctionParameter { name: "state", type_name: "usize" }, FunctionParameter { name: "scene", type_name: "& mut Scene" }], return_type: None }, ]),
        ("Color", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "red", type_name: "f32" }, FunctionParameter { name: "green", type_name: "f32" }, FunctionParameter { name: "blue", type_name: "f32" }, FunctionParameter { name: "alpha", type_name: "f32" }], return_type: Some("Self") }, FunctionSignature { name: "from_rgba", parameters: &[FunctionParameter { name: "d", type_name: "[f32 ; 4]" }], return_type: Some("Self") }, FunctionSignature { name: "from_hex", parameters: &[FunctionParameter { name: "hex", type_name: "& str" }], return_type: Some("Result < Self , csscolorparser :: ParseColorError >") }, FunctionSignature { name: "from_rgb_alpha", parameters: &[FunctionParameter { name: "d", type_name: "[f32 ; 3]" }, FunctionParameter { name: "alpha", type_name: "f32" }], return_type: Some("Self") }, FunctionSignature { name: "from_css", parameters: &[FunctionParameter { name: "color", type_name: "& str" }], return_type: Some("Result < Self , csscolorparser :: ParseColorError >") }, FunctionSignature { name: "green", parameters: &[], return_type: Some("f32") }, FunctionSignature { name: "blue", parameters: &[], return_type: Some("f32") }, FunctionSignature { name: "alpha", parameters: &[], return_type: Some("f32") }, FunctionSignature { name: "red", parameters: &[], return_type: Some("f32") }, FunctionSignature { name: "into_vec4_gamma", parameters: &[], return_type: Some("[f32 ; 4]") }, FunctionSignature { name: "into_vec4", parameters: &[], return_type: Some("[f32 ; 4]") }, ]),
        ("SolidConfig", &[]),
        ("SceneState", &[FunctionSignature { name: "perspective", parameters: &[FunctionParameter { name: "camera_node", type_name: "NodeId" }], return_type: Some("Camera") }, FunctionSignature { name: "get", parameters: &[FunctionParameter { name: "entity", type_name: "ObjectId" }], return_type: Some("Result < C :: Ref , hecs :: ComponentError >") }, FunctionSignature { name: "add_target", parameters: &[FunctionParameter { name: "target", type_name: "Target" }], return_type: None }, FunctionSignature { name: "camera", parameters: &[], return_type: Some("components :: camera :: Camera") }, FunctionSignature { name: "add", parameters: &[FunctionParameter { name: "object", type_name: "& mut impl SceneObjectEntry" }], return_type: Some("ObjectId") }, FunctionSignature { name: "insert", parameters: &[FunctionParameter { name: "entity", type_name: "ObjectId" }, FunctionParameter { name: "component", type_name: "C" }], return_type: Some("Result < () , hecs :: NoSuchEntity >") }, FunctionSignature { name: "query", parameters: &[], return_type: Some("hecs :: QueryBorrow < '_ , Q >") }, FunctionSignature { name: "get_global_transforms", parameters: &[], return_type: Some("GlobalTransforms") }, ]),
        ("Frame", &[FunctionSignature { name: "should_present", parameters: &[], return_type: Some("bool") }, FunctionSignature { name: "present", parameters: &[], return_type: None }, ]),
        ("WindowTarget", &[]),
        ("Controller", &[FunctionSignature { name: "handle_event", parameters: &[FunctionParameter { name: "event", type_name: "& WindowEvent" }], return_type: None }, FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "speed", type_name: "f32" }], return_type: Some("Self") }, FunctionSignature { name: "update_position", parameters: &[FunctionParameter { name: "_node", type_name: "& mut Node" }], return_type: None }, ]),
        ("asset_gltf.rs", &[FunctionSignature { name: "load_gltf", parameters: &[FunctionParameter { name: "path", type_name: "impl AsRef < Path >" }, FunctionParameter { name: "scene", type_name: "& mut crate :: Scene" }, FunctionParameter { name: "global_parent", type_name: "crate :: NodeId" }, FunctionParameter { name: "renderer", type_name: "& mut crate :: Renderer" }], return_type: Some("Module") }, ]),
        ("renderer_resources_sampler.rs", &[FunctionSignature { name: "create_default_sampler", parameters: &[FunctionParameter { name: "device", type_name: "& wgpu :: Device" }], return_type: Some("wgpu :: Sampler") }, FunctionSignature { name: "create_sampler", parameters: &[FunctionParameter { name: "device", type_name: "& wgpu :: Device" }, FunctionParameter { name: "options", type_name: "SamplerOptions" }], return_type: Some("wgpu :: Sampler") }, ]),
        ("WindowState", &[]),
        ("Internal", &[]),
        ("LocalTransform", &[FunctionSignature { name: "to_transform", parameters: &[], return_type: Some("Transform") }, FunctionSignature { name: "inverse_matrix", parameters: &[], return_type: Some("mint :: ColumnMatrix4 < f32 >") }, ]),
        ("Texture", &[FunctionSignature { name: "from_wgpu_texture", parameters: &[FunctionParameter { name: "renderer", type_name: "& crate :: renderer :: Renderer" }, FunctionParameter { name: "texture", type_name: "wgpu :: Texture" }], return_type: Some("Self") }, FunctionSignature { name: "create_depth_texture", parameters: &[FunctionParameter { name: "renderer", type_name: "& crate :: renderer :: Renderer" }, FunctionParameter { name: "size", type_name: "wgpu :: Extent3d" }], return_type: Some("Self") }, FunctionSignature { name: "create_target_texture", parameters: &[FunctionParameter { name: "renderer", type_name: "& Renderer" }, FunctionParameter { name: "size", type_name: "wgpu :: Extent3d" }], return_type: Some("Self") }, FunctionSignature { name: "from_image", parameters: &[FunctionParameter { name: "renderer", type_name: "& crate :: renderer :: Renderer" }, FunctionParameter { name: "image", type_name: "& image :: DynamicImage" }], return_type: Some("Result < Self , Error >") }, FunctionSignature { name: "write_data_to_texture", parameters: &[FunctionParameter { name: "renderer", type_name: "& crate :: renderer :: Renderer" }, FunctionParameter { name: "origin_image", type_name: "RgbaImage" }, FunctionParameter { name: "target_texture", type_name: "& wgpu :: Texture" }, FunctionParameter { name: "size", type_name: "wgpu :: Extent3d" }], return_type: None }, FunctionSignature { name: "from_bytes", parameters: &[FunctionParameter { name: "renderer", type_name: "& crate :: renderer :: Renderer" }, FunctionParameter { name: "bytes", type_name: "& [u8]" }], return_type: Some("Result < Self , Error >") }, ]),
        ("Flat2D", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "renderer", type_name: "& 'r Renderer" }], return_type: Some("Self") }, ]),
        ("Camera", &[FunctionSignature { name: "projection_matrix", parameters: &[FunctionParameter { name: "aspect", type_name: "f32" }], return_type: Some("mint :: ColumnMatrix4 < f32 >") }, ]),
        ("SpriteMap", &[FunctionSignature { name: "at", parameters: &[FunctionParameter { name: "index", type_name: "mint :: Point2 < usize >" }], return_type: Some("crate :: UvRange") }, ]),
        ("MeshBuilder", &[FunctionSignature { name: "name", parameters: &[FunctionParameter { name: "name", type_name: "& str" }], return_type: Some("& 's mut Self") }, FunctionSignature { name: "build", parameters: &[], return_type: Some("MeshPrototype") }, FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "renderer", type_name: "& 'r mut renderer :: Renderer" }], return_type: Some("Self") }, FunctionSignature { name: "radius", parameters: &[FunctionParameter { name: "radius", type_name: "f32" }], return_type: Some("& mut Self") }, FunctionSignature { name: "index", parameters: &[FunctionParameter { name: "data", type_name: "& [u16]" }], return_type: Some("& 's mut Self") }, FunctionSignature { name: "vertex", parameters: &[FunctionParameter { name: "data", type_name: "& [T]" }], return_type: Some("& 's mut Self") }, ]),
        ("AppState", &[FunctionSignature { name: "add_window", parameters: &[FunctionParameter { name: "window", type_name: "& 'w mut W" }], return_type: None }, FunctionSignature { name: "remove_scene", parameters: &[FunctionParameter { name: "window", type_name: "W" }], return_type: None }, FunctionSignature { name: "windows", parameters: &[], return_type: Some("MutexGuard < '_ , Windows >") }, FunctionSignature { name: "renderer", parameters: &[], return_type: Some("MutexGuard < '_ , Renderer >") }, FunctionSignature { name: "scenes", parameters: &[], return_type: Some("MutexGuard < '_ , Windows >") }, FunctionSignature { name: "remove_window", parameters: &[FunctionParameter { name: "window", type_name: "W" }], return_type: None }, ]),
        ("Renderable", &[FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "mesh_id", type_name: "MeshId" }], return_type: Some("Self") }, ]),
        ("app_meta.rs", &[FunctionSignature { name: "version_major", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "build_info", parameters: &[], return_type: Some("String") }, FunctionSignature { name: "is_release", parameters: &[], return_type: Some("bool") }, FunctionSignature { name: "target", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "name", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "repository", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "built_time", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "version", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "print_build_info", parameters: &[], return_type: None }, FunctionSignature { name: "version_patch", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "features", parameters: &[], return_type: Some("& 'static [& 'static str]") }, FunctionSignature { name: "is_debug", parameters: &[], return_type: Some("bool") }, FunctionSignature { name: "host", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "profile", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "version_minor", parameters: &[], return_type: Some("& 'static str") }, FunctionSignature { name: "description", parameters: &[], return_type: Some("& 'static str") }, ]),
        ("NodeId", &[FunctionSignature { name: "as_usize", parameters: &[], return_type: Some("usize") }, FunctionSignature { name: "as_u32", parameters: &[], return_type: Some("u32") }, FunctionSignature { name: "root", parameters: &[], return_type: Some("Self") }, ]),
        ("Position", &[]),
        ("Scene", &[FunctionSignature { name: "new_renderable", parameters: &[FunctionParameter { name: "prototype", type_name: "& MeshPrototype" }], return_type: Some("SceneObject < Renderable >") }, FunctionSignature { name: "new", parameters: &[], return_type: Some("Self") }, FunctionSignature { name: "add_target", parameters: &[FunctionParameter { name: "target", type_name: "Target" }], return_type: None }, FunctionSignature { name: "camera", parameters: &[], return_type: Some("components :: camera :: Camera") }, FunctionSignature { name: "get_global_transforms", parameters: &[], return_type: Some("GlobalTransforms") }, FunctionSignature { name: "new_sprite", parameters: &[FunctionParameter { name: "image", type_name: "TextureId" }], return_type: Some("SceneObject < Sprite >") }, FunctionSignature { name: "state_mut", parameters: &[], return_type: Some("RwLockWriteGuard < '_ , SceneState >") }, FunctionSignature { name: "new_empty", parameters: &[], return_type: Some("SceneObject < () >") }, FunctionSignature { name: "state", parameters: &[], return_type: Some("RwLockReadGuard < '_ , SceneState >") }, FunctionSignature { name: "add", parameters: &[FunctionParameter { name: "object", type_name: "& mut impl SceneObjectEntry" }], return_type: Some("ObjectId") }, FunctionSignature { name: "new_object", parameters: &[FunctionParameter { name: "object", type_name: "T" }], return_type: Some("SceneObject < T >") }, ]),
        ("Transform", &[]),
        ("Window", &[FunctionSignature { name: "set_fullscreen", parameters: &[FunctionParameter { name: "fullscreen", type_name: "bool" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_min_size", parameters: &[FunctionParameter { name: "size", type_name: "Option < (u32 , u32) >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_resizable", parameters: &[FunctionParameter { name: "resizable", type_name: "bool" }], return_type: Some("& mut Self") }, FunctionSignature { name: "new", parameters: &[FunctionParameter { name: "options", type_name: "WindowOptions" }], return_type: Some("Result < Self , winit :: error :: OsError >") }, FunctionSignature { name: "set_size", parameters: &[FunctionParameter { name: "size", type_name: "(u32 , u32)" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_max_size", parameters: &[FunctionParameter { name: "size", type_name: "Option < (u32 , u32) >" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_auto_resize", parameters: &[FunctionParameter { name: "auto_resize", type_name: "bool" }], return_type: Some("& mut Self") }, FunctionSignature { name: "set_decorations", parameters: &[FunctionParameter { name: "decorations", type_name: "bool" }], return_type: Some("& mut Self") }, FunctionSignature { name: "run", parameters: &[], return_type: None }, FunctionSignature { name: "on", parameters: &[FunctionParameter { name: "event_name", type_name: "& str" }, FunctionParameter { name: "callback", type_name: "impl CallbackFn < Event > + 'static" }], return_type: None }, FunctionSignature { name: "set_title", parameters: &[FunctionParameter { name: "title", type_name: "& str" }], return_type: Some("& mut Self") }, FunctionSignature { name: "call", parameters: &[FunctionParameter { name: "event_name", type_name: "& str" }, FunctionParameter { name: "event", type_name: "Event" }], return_type: None }, ]),
        ("AppOptions", &[]),
        ("asset_obj.rs", &[FunctionSignature { name: "load_obj", parameters: &[FunctionParameter { name: "path", type_name: "impl AsRef < Path >" }, FunctionParameter { name: "scene", type_name: "& mut crate :: Scene" }, FunctionParameter { name: "node", type_name: "crate :: NodeId" }, FunctionParameter { name: "renderer", type_name: "& mut crate :: Renderer" }], return_type: Some("fxhash :: FxHashMap < String , crate :: ObjectId >") }, ]),
        ("RealConfig", &[]),
    ],
};
