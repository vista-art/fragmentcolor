(function() {var implementors = {
"euclid":[["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T, Output = T&gt;, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Vector2D.html\" title=\"struct euclid::Vector2D\">Vector2D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Vector2D.html\" title=\"struct euclid::Vector2D\">Vector2D</a>&lt;T, U&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Size2D.html\" title=\"struct euclid::Size2D\">Size2D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Size2D.html\" title=\"struct euclid::Size2D\">Size2D</a>&lt;T, U&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T, Output = T&gt;, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Vector2D.html\" title=\"struct euclid::Vector2D\">Vector2D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Point2D.html\" title=\"struct euclid::Point2D\">Point2D</a>&lt;T, U&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Size3D.html\" title=\"struct euclid::Size3D\">Size3D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Size3D.html\" title=\"struct euclid::Size3D\">Size3D</a>&lt;T, U&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Length.html\" title=\"struct euclid::Length\">Length</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Length.html\" title=\"struct euclid::Length\">Length</a>&lt;T, U&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;T&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Angle.html\" title=\"struct euclid::Angle\">Angle</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Angle.html\" title=\"struct euclid::Angle\">Angle</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>, Src, Dst&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Translation2D.html\" title=\"struct euclid::Translation2D\">Translation2D</a>&lt;T, Dst, Dst&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Translation2D.html\" title=\"struct euclid::Translation2D\">Translation2D</a>&lt;T, Src, Dst&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Size3D.html\" title=\"struct euclid::Size3D\">Size3D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Point3D.html\" title=\"struct euclid::Point3D\">Point3D</a>&lt;T, U&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>, Src, Dst&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Translation3D.html\" title=\"struct euclid::Translation3D\">Translation3D</a>&lt;T, Dst, Dst&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Translation3D.html\" title=\"struct euclid::Translation3D\">Translation3D</a>&lt;T, Src, Dst&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Size2D.html\" title=\"struct euclid::Size2D\">Size2D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Point2D.html\" title=\"struct euclid::Point2D\">Point2D</a>&lt;T, U&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T, Output = T&gt;, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Vector3D.html\" title=\"struct euclid::Vector3D\">Vector3D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Point3D.html\" title=\"struct euclid::Point3D\">Point3D</a>&lt;T, U&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.Add.html\" title=\"trait core::ops::arith::Add\">Add</a>&lt;T, Output = T&gt;, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.Vector3D.html\" title=\"struct euclid::Vector3D\">Vector3D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.Vector3D.html\" title=\"struct euclid::Vector3D\">Vector3D</a>&lt;T, U&gt;"],["impl&lt;T, U&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"euclid/struct.SideOffsets2D.html\" title=\"struct euclid::SideOffsets2D\">SideOffsets2D</a>&lt;T, U&gt;&gt; for <a class=\"struct\" href=\"euclid/struct.SideOffsets2D.html\" title=\"struct euclid::SideOffsets2D\">SideOffsets2D</a>&lt;T, U&gt;<span class=\"where fmt-newline\">where\n    T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;T&gt;,</span>"]],
"glam":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/i64/struct.I64Vec3.html\" title=\"struct glam::i64::I64Vec3\">I64Vec3</a>&gt; for <a class=\"struct\" href=\"glam/i64/struct.I64Vec3.html\" title=\"struct glam::i64::I64Vec3\">I64Vec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f32/struct.Vec2.html\" title=\"struct glam::f32::Vec2\">Vec2</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Vec2.html\" title=\"struct glam::f32::Vec2\">Vec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.i32.html\">i32</a>&gt; for <a class=\"struct\" href=\"glam/i32/struct.IVec3.html\" title=\"struct glam::i32::IVec3\">IVec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.u32.html\">u32</a>&gt; for <a class=\"struct\" href=\"glam/u32/struct.UVec4.html\" title=\"struct glam::u32::UVec4\">UVec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/u32/struct.UVec3.html\" title=\"struct glam::u32::UVec3\">UVec3</a>&gt; for <a class=\"struct\" href=\"glam/u32/struct.UVec3.html\" title=\"struct glam::u32::UVec3\">UVec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f64/struct.DMat3.html\" title=\"struct glam::f64::DMat3\">DMat3</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DMat3.html\" title=\"struct glam::f64::DMat3\">DMat3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f64/struct.DVec3.html\" title=\"struct glam::f64::DVec3\">DVec3</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DVec3.html\" title=\"struct glam::f64::DVec3\">DVec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f64/struct.DMat2.html\" title=\"struct glam::f64::DMat2\">DMat2</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DMat2.html\" title=\"struct glam::f64::DMat2\">DMat2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/u64/struct.U64Vec2.html\" title=\"struct glam::u64::U64Vec2\">U64Vec2</a>&gt; for <a class=\"struct\" href=\"glam/u64/struct.U64Vec2.html\" title=\"struct glam::u64::U64Vec2\">U64Vec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.f32.html\">f32</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Vec4.html\" title=\"struct glam::f32::Vec4\">Vec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/i64/struct.I64Vec2.html\" title=\"struct glam::i64::I64Vec2\">I64Vec2</a>&gt; for <a class=\"struct\" href=\"glam/i64/struct.I64Vec2.html\" title=\"struct glam::i64::I64Vec2\">I64Vec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.u64.html\">u64</a>&gt; for <a class=\"struct\" href=\"glam/u64/struct.U64Vec4.html\" title=\"struct glam::u64::U64Vec4\">U64Vec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f32/struct.Vec3A.html\" title=\"struct glam::f32::Vec3A\">Vec3A</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Vec3A.html\" title=\"struct glam::f32::Vec3A\">Vec3A</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.f32.html\">f32</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Vec3.html\" title=\"struct glam::f32::Vec3\">Vec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.f64.html\">f64</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DVec3.html\" title=\"struct glam::f64::DVec3\">DVec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.f32.html\">f32</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Vec2.html\" title=\"struct glam::f32::Vec2\">Vec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/i64/struct.I64Vec4.html\" title=\"struct glam::i64::I64Vec4\">I64Vec4</a>&gt; for <a class=\"struct\" href=\"glam/i64/struct.I64Vec4.html\" title=\"struct glam::i64::I64Vec4\">I64Vec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f32/struct.Mat2.html\" title=\"struct glam::f32::Mat2\">Mat2</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Mat2.html\" title=\"struct glam::f32::Mat2\">Mat2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.f64.html\">f64</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DVec4.html\" title=\"struct glam::f64::DVec4\">DVec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f32/struct.Vec4.html\" title=\"struct glam::f32::Vec4\">Vec4</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Vec4.html\" title=\"struct glam::f32::Vec4\">Vec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/i32/struct.IVec4.html\" title=\"struct glam::i32::IVec4\">IVec4</a>&gt; for <a class=\"struct\" href=\"glam/i32/struct.IVec4.html\" title=\"struct glam::i32::IVec4\">IVec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f64/struct.DVec2.html\" title=\"struct glam::f64::DVec2\">DVec2</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DVec2.html\" title=\"struct glam::f64::DVec2\">DVec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/u32/struct.UVec4.html\" title=\"struct glam::u32::UVec4\">UVec4</a>&gt; for <a class=\"struct\" href=\"glam/u32/struct.UVec4.html\" title=\"struct glam::u32::UVec4\">UVec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f64/struct.DMat4.html\" title=\"struct glam::f64::DMat4\">DMat4</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DMat4.html\" title=\"struct glam::f64::DMat4\">DMat4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/u64/struct.U64Vec3.html\" title=\"struct glam::u64::U64Vec3\">U64Vec3</a>&gt; for <a class=\"struct\" href=\"glam/u64/struct.U64Vec3.html\" title=\"struct glam::u64::U64Vec3\">U64Vec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.u32.html\">u32</a>&gt; for <a class=\"struct\" href=\"glam/u32/struct.UVec2.html\" title=\"struct glam::u32::UVec2\">UVec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f32/struct.Mat3A.html\" title=\"struct glam::f32::Mat3A\">Mat3A</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Mat3A.html\" title=\"struct glam::f32::Mat3A\">Mat3A</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/i32/struct.IVec3.html\" title=\"struct glam::i32::IVec3\">IVec3</a>&gt; for <a class=\"struct\" href=\"glam/i32/struct.IVec3.html\" title=\"struct glam::i32::IVec3\">IVec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.i64.html\">i64</a>&gt; for <a class=\"struct\" href=\"glam/i64/struct.I64Vec2.html\" title=\"struct glam::i64::I64Vec2\">I64Vec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/u32/struct.UVec2.html\" title=\"struct glam::u32::UVec2\">UVec2</a>&gt; for <a class=\"struct\" href=\"glam/u32/struct.UVec2.html\" title=\"struct glam::u32::UVec2\">UVec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.f64.html\">f64</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DVec2.html\" title=\"struct glam::f64::DVec2\">DVec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.i32.html\">i32</a>&gt; for <a class=\"struct\" href=\"glam/i32/struct.IVec2.html\" title=\"struct glam::i32::IVec2\">IVec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.i64.html\">i64</a>&gt; for <a class=\"struct\" href=\"glam/i64/struct.I64Vec4.html\" title=\"struct glam::i64::I64Vec4\">I64Vec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.u64.html\">u64</a>&gt; for <a class=\"struct\" href=\"glam/u64/struct.U64Vec2.html\" title=\"struct glam::u64::U64Vec2\">U64Vec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f32/struct.Mat4.html\" title=\"struct glam::f32::Mat4\">Mat4</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Mat4.html\" title=\"struct glam::f32::Mat4\">Mat4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.i32.html\">i32</a>&gt; for <a class=\"struct\" href=\"glam/i32/struct.IVec4.html\" title=\"struct glam::i32::IVec4\">IVec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.u64.html\">u64</a>&gt; for <a class=\"struct\" href=\"glam/u64/struct.U64Vec3.html\" title=\"struct glam::u64::U64Vec3\">U64Vec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.u32.html\">u32</a>&gt; for <a class=\"struct\" href=\"glam/u32/struct.UVec3.html\" title=\"struct glam::u32::UVec3\">UVec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.f32.html\">f32</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Vec3A.html\" title=\"struct glam::f32::Vec3A\">Vec3A</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f64/struct.DVec4.html\" title=\"struct glam::f64::DVec4\">DVec4</a>&gt; for <a class=\"struct\" href=\"glam/f64/struct.DVec4.html\" title=\"struct glam::f64::DVec4\">DVec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/i32/struct.IVec2.html\" title=\"struct glam::i32::IVec2\">IVec2</a>&gt; for <a class=\"struct\" href=\"glam/i32/struct.IVec2.html\" title=\"struct glam::i32::IVec2\">IVec2</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f32/struct.Mat3.html\" title=\"struct glam::f32::Mat3\">Mat3</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Mat3.html\" title=\"struct glam::f32::Mat3\">Mat3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.i64.html\">i64</a>&gt; for <a class=\"struct\" href=\"glam/i64/struct.I64Vec3.html\" title=\"struct glam::i64::I64Vec3\">I64Vec3</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/u64/struct.U64Vec4.html\" title=\"struct glam::u64::U64Vec4\">U64Vec4</a>&gt; for <a class=\"struct\" href=\"glam/u64/struct.U64Vec4.html\" title=\"struct glam::u64::U64Vec4\">U64Vec4</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"glam/f32/struct.Vec3.html\" title=\"struct glam::f32::Vec3\">Vec3</a>&gt; for <a class=\"struct\" href=\"glam/f32/struct.Vec3.html\" title=\"struct glam::f32::Vec3\">Vec3</a>"]],
"num_rational":[["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"num_integer/trait.Integer.html\" title=\"trait num_integer::Integer\">Integer</a> + <a class=\"trait\" href=\"num_traits/trait.NumAssign.html\" title=\"trait num_traits::NumAssign\">NumAssign</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;T&gt; for <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;"],["impl&lt;'a, T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"num_integer/trait.Integer.html\" title=\"trait num_integer::Integer\">Integer</a> + <a class=\"trait\" href=\"num_traits/trait.NumAssign.html\" title=\"trait num_traits::NumAssign\">NumAssign</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.reference.html\">&amp;'a T</a>&gt; for <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;"],["impl&lt;'a, T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"num_integer/trait.Integer.html\" title=\"trait num_integer::Integer\">Integer</a> + <a class=\"trait\" href=\"num_traits/trait.NumAssign.html\" title=\"trait num_traits::NumAssign\">NumAssign</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;&amp;'a <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"num_integer/trait.Integer.html\" title=\"trait num_integer::Integer\">Integer</a> + <a class=\"trait\" href=\"num_traits/trait.NumAssign.html\" title=\"trait num_traits::NumAssign\">NumAssign</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;&gt; for <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;"]],
"objc2":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;&amp;<a class=\"struct\" href=\"objc2/foundation/struct.NSString.html\" title=\"struct objc2::foundation::NSString\">NSString</a>&gt; for <a class=\"struct\" href=\"objc2/foundation/struct.NSMutableString.html\" title=\"struct objc2::foundation::NSMutableString\">NSMutableString</a>"]],
"ppv_lite86":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"ppv_lite86/generic/struct.u32x4_generic.html\" title=\"struct ppv_lite86::generic::u32x4_generic\">u32x4_generic</a>&gt; for <a class=\"struct\" href=\"ppv_lite86/generic/struct.u32x4_generic.html\" title=\"struct ppv_lite86::generic::u32x4_generic\">u32x4_generic</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"ppv_lite86/generic/struct.u64x2_generic.html\" title=\"struct ppv_lite86::generic::u64x2_generic\">u64x2_generic</a>&gt; for <a class=\"struct\" href=\"ppv_lite86/generic/struct.u64x2_generic.html\" title=\"struct ppv_lite86::generic::u64x2_generic\">u64x2_generic</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/arith/trait.AddAssign.html\" title=\"trait core::ops::arith::AddAssign\">AddAssign</a>&lt;<a class=\"struct\" href=\"ppv_lite86/generic/struct.u128x1_generic.html\" title=\"struct ppv_lite86::generic::u128x1_generic\">u128x1_generic</a>&gt; for <a class=\"struct\" href=\"ppv_lite86/generic/struct.u128x1_generic.html\" title=\"struct ppv_lite86::generic::u128x1_generic\">u128x1_generic</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()