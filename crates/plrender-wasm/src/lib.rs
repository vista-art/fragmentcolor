#![cfg(wasm)]

#[cfg(not(wasm))]
compile_error!("This library only supports Wasm target!");

mod window;

pub use plr::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = PLRender)]
pub struct JsPLRender;

#[wasm_bindgen(js_class = App)]
impl JsPLRender {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Self {
        Self {
            inner: App::new(AppOptions {
                device_limits: "webgl2",
                ..default::Default()
            }),
        }
    }
}

pub struct JsPLRender {}

pub struct JsScene {
    inner: Scene,
}

#[wasm_bindgen(js_class = Scene)]
impl JsScene {
    #[wasm_bindgen(constructor)]
    pub fn new(app: &WasmApp) -> Self {
        Self {
            inner: window::init_window(app.inner.event_loop().window_target()),
        }
    }
}

#[wasm_bindgen(js_class = ShapeBuilder)]
impl JsShapeBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ShapeBuilder::default(),
        }
    }
}

#[wasm_bindgen(js_class = PLRender)]
impl JsPLRender {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: PLRender::default(),
        }
    }
}

#[wasm_bindgen(js_class = Primitive)]
impl JsPrimitive {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Primitive::default(),
        }
    }
}

#[wasm_bindgen(js_class = Scenes)]
impl JsScenes {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Scenes::default(),
        }
    }
}

#[wasm_bindgen(js_class = MeshId)]
impl JsMeshId {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: MeshId::default(),
        }
    }
}

#[wasm_bindgen(js_class = Sphere)]
impl JsSphere {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Sphere::default(),
        }
    }
}

#[wasm_bindgen(js_class = Extra)]
impl JsExtra {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Extra::default(),
        }
    }
}

#[wasm_bindgen(js_class = Windows)]
impl JsWindows {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Windows::default(),
        }
    }
}

#[wasm_bindgen(js_class = RendererOptions)]
impl JsRendererOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RendererOptions::default(),
        }
    }
}

#[wasm_bindgen(js_class = Vertex)]
impl JsVertex {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Vertex::default(),
        }
    }
}

#[wasm_bindgen(js_class = SceneId)]
impl JsSceneId {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SceneId::default(),
        }
    }
}

#[wasm_bindgen(js_class = LightOptions)]
impl JsLightOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: LightOptions::default(),
        }
    }
}

#[wasm_bindgen(js_class = Plane)]
impl JsPlane {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Plane::default(),
        }
    }
}

#[wasm_bindgen(js_class = Sprite)]
impl JsSprite {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Sprite::default(),
        }
    }
}

#[wasm_bindgen(js_class = Vec2or3)]
impl JsVec2or3 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Vec2or3::default(),
        }
    }
}

#[wasm_bindgen(js_class = TextureId)]
impl JsTextureId {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: TextureId::default(),
        }
    }
}

#[wasm_bindgen(js_class = QuadVertex)]
impl JsQuadVertex {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: QuadVertex::default(),
        }
    }
}

#[wasm_bindgen(js_class = Controller)]
impl JsController {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Controller::default(),
        }
    }
}

#[wasm_bindgen(js_class = VertexData)]
impl JsVertexData {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: VertexData::default(),
        }
    }
}

#[wasm_bindgen(js_class = Empty)]
impl JsEmpty {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Empty::default(),
        }
    }
}

#[wasm_bindgen(js_class = TextureCoordinates)]
impl JsTextureCoordinates {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: TextureCoordinates::default(),
        }
    }
}

#[wasm_bindgen(js_class = SceneObject)]
impl JsSceneObject {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SceneObject::default(),
        }
    }
}

#[wasm_bindgen(js_class = TransformId)]
impl JsTransformId {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: TransformId::default(),
        }
    }
}

#[wasm_bindgen(js_class = Mesh)]
impl JsMesh {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Mesh::default(),
        }
    }
}

#[wasm_bindgen(js_class = Scene)]
impl JsScene {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Scene::default(),
        }
    }
}

#[wasm_bindgen(js_class = Color)]
impl JsColor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Color::default(),
        }
    }
}

#[wasm_bindgen(js_class = AppOptions)]
impl JsAppOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: AppOptions::default(),
        }
    }
}

#[wasm_bindgen(js_class = Position)]
impl JsPosition {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Position::default(),
        }
    }
}

#[wasm_bindgen(js_class = LocalTransform)]
impl JsLocalTransform {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: LocalTransform::default(),
        }
    }
}

#[wasm_bindgen(js_class = Target)]
impl JsTarget {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Target::default(),
        }
    }
}

#[wasm_bindgen(js_class = ObjectBuilder)]
impl JsObjectBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ObjectBuilder::default(),
        }
    }
}

#[wasm_bindgen(js_class = Camera)]
impl JsCamera {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Camera::default(),
        }
    }
}

#[wasm_bindgen(js_class = RenderTargets)]
impl JsRenderTargets {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RenderTargets::default(),
        }
    }
}

#[wasm_bindgen(js_class = Frame)]
impl JsFrame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Frame::default(),
        }
    }
}

#[wasm_bindgen(js_class = Normal)]
impl JsNormal {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Normal::default(),
        }
    }
}

#[wasm_bindgen(js_class = Quad)]
impl JsQuad {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Quad::default(),
        }
    }
}

#[wasm_bindgen(js_class = WindowOptions)]
impl JsWindowOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: WindowOptions::default(),
        }
    }
}

#[wasm_bindgen(js_class = MeshData)]
impl JsMeshData {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: MeshData::default(),
        }
    }
}

#[wasm_bindgen(js_class = TextureTarget)]
impl JsTextureTarget {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: TextureTarget::default(),
        }
    }
}

#[wasm_bindgen(js_class = Projection)]
impl JsProjection {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Projection::default(),
        }
    }
}

#[wasm_bindgen(js_class = WindowState)]
impl JsWindowState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: WindowState::default(),
        }
    }
}

#[wasm_bindgen(js_class = SceneState)]
impl JsSceneState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: SceneState::default(),
        }
    }
}

#[wasm_bindgen(js_class = BuiltMesh)]
impl JsBuiltMesh {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: BuiltMesh::default(),
        }
    }
}

#[wasm_bindgen(js_class = MeshBuilder)]
impl JsMeshBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: MeshBuilder::default(),
        }
    }
}

#[wasm_bindgen(js_class = Light)]
impl JsLight {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Light::default(),
        }
    }
}

#[wasm_bindgen(js_class = F)]
impl JsF {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: F::default(),
        }
    }
}

#[wasm_bindgen(js_class = LightType)]
impl JsLightType {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: LightType::default(),
        }
    }
}

#[wasm_bindgen(js_class = Box)]
impl JsBox {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Box::default(),
        }
    }
}

#[wasm_bindgen(js_class = Resources)]
impl JsResources {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Resources::default(),
        }
    }
}

#[wasm_bindgen(js_class = Canvas)]
impl Canvas {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Window::default(),
        }
    }
}

#[wasm_bindgen(js_class = CameraOptions)]
impl JsCameraOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CameraOptions::default(),
        }
    }
}

#[wasm_bindgen(js_class = Cube)]
impl JsCube {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Cube::default(),
        }
    }
}

#[wasm_bindgen(js_class = Texture)]
impl JsTexture {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Texture::default(),
        }
    }
}
