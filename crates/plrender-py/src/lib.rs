pub use plrender::{
    Camera, Color, Context, Entity, EntityRef, ImageRef, Light, LightBuilder, LightRef,
    MeshBuilder, MeshRef, Node, NodeRef, Pass, Projection, Prototype, Scene, Sprite, SpriteBuilder,
    TargetInfo, TargetRef, UvRange,
};
use plrender_macros::wrap_py;
use pyo3::prelude::*;

wrap_py!(Camera);
wrap_py!(Color);
wrap_py!(Context);
wrap_py!(Entity);
wrap_py!(EntityRef);
wrap_py!(ImageRef);
wrap_py!(Light);
wrap_py!(LightBuilder);
wrap_py!(LightRef);
wrap_py!(MeshBuilder);
wrap_py!(MeshRef);
wrap_py!(Node);
wrap_py!(NodeRef);
wrap_py!(Pass);
wrap_py!(Projection);
wrap_py!(Prototype);
wrap_py!(Scene);
wrap_py!(Sprite);
wrap_py!(SpriteBuilder);
wrap_py!(TargetInfo);
wrap_py!(TargetRef);
wrap_py!(UvRange);
