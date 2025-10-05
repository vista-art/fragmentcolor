use image::{ImageReader, RgbaImage};
use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use fragmentcolor::{Pass, Renderer, Shader, Target};

pub struct Tolerance(pub f64);

impl Default for Tolerance {
    fn default() -> Self {
        Tolerance(0.01)
    }
}

// tests directory as the base
const BASE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");

fn golden_path<P: AsRef<Path>>(path: P) -> PathBuf {
    Path::new(BASE_DIR).join("golden").join(path)
}

fn error_path<P: AsRef<Path>>(path: P) -> PathBuf {
    Path::new(BASE_DIR).join("error").join(path)
}

fn parse_file<P: AsRef<Path>>(path: P) -> image::RgbaImage {
    let data = std::fs::read(path).unwrap();
    ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8()
}

fn save_png(path: &Path, w: u32, h: u32, rgba: &[u8]) -> image::ImageResult<()> {
    let img = image::RgbaImage::from_raw(w, h, rgba.to_vec()).expect("rgba shape");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    img.save(path)
}

fn compare_images(result: &RgbaImage, expected: &RgbaImage, tolerance: Tolerance, name: &str) {
    let tolerance = tolerance.0;
    let similarity = image_compare::rgba_hybrid_compare(result, expected).unwrap_or_else(|_| {
        let _ = fs::create_dir_all(error_path("."));
        let _ = result.save(error_path(format!("{}-result.png", name)));
        let _ = expected.save(error_path(format!("{}-expected.png", name)));
        panic!(
            "\nFailed to compare images of different dimensions: {:?} vs {:?}.\nSee tests/error/{}-(result|expected).png\n",
            result.dimensions(), expected.dimensions(), name
        );
    });

    let difference = 1.0 - similarity.score;
    if difference > tolerance {
        let _ = fs::create_dir_all(error_path("."));
        let _ = result.save(error_path(format!("{}-result.png", name)));
        let _ = expected.save(error_path(format!("{}-expected.png", name)));
        let diff = error_path(format!("{}-diff.png", name));
        similarity.image.to_color_map().save(&diff).unwrap();
        panic!(
            "\nImages differ by {:.2}% (tolerance {:.2}%). See {}\n",
            difference * 100.0,
            tolerance * 100.0,
            diff.display()
        );
    }
}

async fn render_pass_to_rgba(pass: &Pass, size: [u32; 2]) -> RgbaImage {
    let renderer = Renderer::new();
    let target = renderer
        .create_texture_target(size)
        .await
        .expect("texture target");
    renderer.render(pass, &target).expect("render ok");
    let bytes = target.get_image();

    RgbaImage::from_raw(size[0], size[1], bytes).expect("rgba shape")
}

async fn render_shader_to_rgba(shader: &Shader, size: [u32; 2]) -> RgbaImage {
    let pass = Pass::from_shader("snapshot", shader);
    render_pass_to_rgba(&pass, size).await
}

/// GOLDEN © Universal Music Publishing Group
///
/// I was a ghost, I was alone (hah)
/// 어두워진 (hah) 앞길 속에 (haah)
/// Given the throne, I didn't know how to believe
/// I was the queen that I'm meant to be
///
/// I lived two lives, tried to play both sides
/// But I couldn't find my own place
/// Called a problem child 'cause I got too wild
/// But now that's how I'm getting paid 끝없이 on stage
///
/// I'm done hidin', now I'm shinin' like I'm born to be
/// We dreamin' hard, we came so far, now I believe
///
/// We're goin' up, up, up, it's our moment
/// You know together we're glowing
/// Gonna be, gonna be golden
/// Oh, up, up, up with our voices
/// 영원히 깨질 수 없는
/// Gonna be, gonna be golden
///
/// Oh, I'm done hidin', now I'm shinin' like I'm born to be
/// Oh, our time, no fears, no lies
/// That's who we're born to be
///
/// Waited so long to break these walls down
/// To wake up and feel like me
/// Put these patterns all in the past now
/// And finally live like the girl they all see
///
/// No more hiding, I'll be shining like I'm born to be
/// 'Cause we are hunters, voices strong, and I know I believe
///
/// We're goin' up, up, up, it's our moment
/// You know together we're glowing
/// Gonna be, gonna be golden
/// Oh, up, up, up, with our voices
/// 영원히 깨질 수 없는
/// Gonna be, gonna be golden
///
/// Oh, I'm done hidin', now I'm shining like I'm born to be
/// Oh, our time, no fears, no lies
/// That's who we're born to be
///
/// You know we're gonna be, gonna be golden
/// We're gonna be, gonna be
/// Born to be, born to be glowin'
/// 밝게 빛나는 우린
///
/// You know that it's our time, no fears, no lies
/// That's who we're born to be
///
/// Source: LyricFind
///
/// Songwriters:
/// ------------
/// Eun Jae Phoebe Kim / Hee Dong Nam / Hong Jun Park /
/// Jeong Hun Seo / Joong Gyu Kwak / Mark Sonnenblick / Yoo Han Lee
fn gonna_be_gonna_be_golden() -> bool {
    env::var("GOLDEN").ok().is_some()
}

#[test]
fn snapshot_hello_triangle() {
    pollster::block_on(async move {
        let size = [128u32, 128u32];
        // Deterministic fullscreen triangle shader inline
        let wgsl = r#"
struct VOut { @builtin(position) pos: vec4<f32> };
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
  var p = array<vec2<f32>, 3>(vec2<f32>(-1.,-1.), vec2<f32>(3.,-1.), vec2<f32>(-1.,3.));
  var out: VOut;
  out.pos = vec4<f32>(p[i], 0., 1.);
  return out;
}
@fragment
fn fs_main(_v: VOut) -> @location(0) vec4<f32> { return vec4<f32>(1.,0.,0.,1.); }
"#;
        let shader = Shader::new(wgsl).expect("shader");

        let img = render_shader_to_rgba(&shader, size).await;
        let name = "fullscreen_triangle";
        let golden = golden_path(format!("{}.png", name));

        if gonna_be_gonna_be_golden() {
            save_png(&golden, size[0], size[1], &img).expect("save golden");
            return;
        }

        if !golden.exists() {
            panic!(
                "Golden missing at {}. Run with GOLDEN=1 to create it.",
                golden.display()
            );
        }
        let expected = parse_file(&golden);
        compare_images(&img, &expected, Tolerance::default(), name);
    })
}
