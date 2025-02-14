use super::ShaderValue;

// @TODO write macro for them
impl From<f32> for ShaderValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<[f32; 1]> for ShaderValue {
    fn from(value: [f32; 1]) -> Self {
        Self::Float(value[0])
    }
}

impl From<i32> for ShaderValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<[i32; 1]> for ShaderValue {
    fn from(value: [i32; 1]) -> Self {
        Self::Int(value[0])
    }
}

impl From<u32> for ShaderValue {
    fn from(value: u32) -> Self {
        Self::UInt(value)
    }
}

impl From<[u32; 1]> for ShaderValue {
    fn from(value: [u32; 1]) -> Self {
        Self::UInt(value[0])
    }
}

impl From<[f32; 2]> for ShaderValue {
    fn from(value: [f32; 2]) -> Self {
        Self::Vec2(value)
    }
}

impl From<(f32, f32)> for ShaderValue {
    fn from(value: (f32, f32)) -> Self {
        Self::Vec2([value.0, value.1])
    }
}

impl From<glam::Vec2> for ShaderValue {
    fn from(v: glam::Vec2) -> Self {
        Self::Vec2(v.to_array())
    }
}

impl From<[f32; 3]> for ShaderValue {
    fn from(value: [f32; 3]) -> Self {
        Self::Vec3(value)
    }
}

impl From<(f32, f32, f32)> for ShaderValue {
    fn from(value: (f32, f32, f32)) -> Self {
        Self::Vec3([value.0, value.1, value.2])
    }
}

impl From<glam::Vec3> for ShaderValue {
    fn from(v: glam::Vec3) -> Self {
        Self::Vec3(v.to_array())
    }
}

impl From<[f32; 4]> for ShaderValue {
    fn from(value: [f32; 4]) -> Self {
        Self::Vec4(value)
    }
}

impl From<(f32, f32, f32, f32)> for ShaderValue {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::Vec4([value.0, value.1, value.2, value.3])
    }
}

impl From<glam::Vec4> for ShaderValue {
    fn from(v: glam::Vec4) -> Self {
        Self::Vec4(v.to_array())
    }
}
