//! Runs a tutorial program in a window.
//!
//! It calls the step's `setup` once, then its `frame` on every redraw
//! with the seconds since the window opened. `FC_TUTORIAL_FRAMES=n` skips the
//! window: it renders `n` frames into a 1280 by 720 texture, prints the average
//! frame time, and fails if less than 1% of the last frame differs from its corner pixel.
//! `FC_TUTORIAL_IMAGE=path` also saves that frame as a PNG.

use anyhow::{Result, bail};
use fragmentcolor::{
    App, FragmentColorError, RenderTarget, Renderer, RendererError, SetupResult, SurfaceError,
    Target,
};
use image::ColorType;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

/// A step's `frame` with its target type fixed to the one `App` hands out.
pub type Frame<State> = fn(&State, &Renderer, &RenderTarget, f32) -> Result<(), FragmentColorError>;

const STATE: &str = "state";
const CHECK_SIZE: [u32; 2] = [1280, 720];
const SYNC_EVERY: u32 = 10;

pub fn run<State, Setup>(setup: Setup, frame: Frame<State>)
where
    State: Send + Sync + 'static,
    Setup: AsyncFnOnce(&Renderer) -> Result<State, FragmentColorError> + 'static,
{
    env_logger::init();
    let Some(frames) = read_frames() else {
        return open_window(setup, frame);
    };
    if let Err(error) = pollster::block_on(check(setup, frame, frames)) {
        eprintln!("check failed: {error}");
        std::process::exit(1);
    }
}

fn read_frames() -> Option<u32> {
    std::env::var("FC_TUTORIAL_FRAMES").ok()?.parse().ok()
}

fn open_window<State, Setup>(setup: Setup, frame: Frame<State>)
where
    State: Send + Sync + 'static,
    Setup: AsyncFnOnce(&Renderer) -> Result<State, FragmentColorError> + 'static,
{
    let start = Instant::now();
    let mut app = App::new(Renderer::new());
    app.on_start(move |app, windows| pollster::block_on(start_app(app, windows, setup)))
        .on_resize(|app, size| app.resize([size.width, size.height]))
        .on_redraw_requested(move |app| draw(app, frame, start.elapsed().as_secs_f32()));
    app.run();
}

async fn start_app<State, Setup>(app: &App, windows: Vec<Arc<Window>>, setup: Setup) -> SetupResult
where
    State: Send + Sync + 'static,
    Setup: AsyncFnOnce(&Renderer) -> Result<State, FragmentColorError>,
{
    for window in windows {
        let target = app.get_renderer().create_target(window.clone()).await?;
        app.add_target(window.id(), target);
    }
    let state = setup(app.get_renderer()).await?;
    app.add_state(STATE, state);
    Ok(())
}

fn draw<State>(app: &App, frame: Frame<State>, time: f32)
where
    State: Send + Sync + 'static,
{
    let Some(state) = app.get_state::<State>(STATE) else {
        return;
    };
    let renderer = app.get_renderer();
    let drawn = app.with_target(app.primary_window_id(), |target| {
        frame(&state, renderer, target, time)
    });
    match drawn {
        // A hidden or minimized window has nothing to draw on, so the frame is skipped.
        Some(Err(FragmentColorError::Renderer(RendererError::SurfaceError(
            SurfaceError::Occluded,
        )))) => {}
        Some(Err(error)) => {
            eprintln!("frame failed: {error}");
            std::process::exit(1);
        }
        _ => {}
    }
}

async fn check<State, Setup>(setup: Setup, frame: Frame<State>, frames: u32) -> Result<()>
where
    Setup: AsyncFnOnce(&Renderer) -> Result<State, FragmentColorError>,
{
    if frames == 0 {
        bail!("FC_TUTORIAL_FRAMES needs at least 1 frame to check");
    }
    let renderer = Renderer::new();
    let texture = renderer.create_texture_target(CHECK_SIZE).await?;
    let target = RenderTarget::from(texture);
    let state = setup(&renderer).await?;
    // Reading the image back waits for the GPU, so the clock covers every frame.
    // Waiting every few frames keeps the queue shorter than the readback's timeout.
    let start = Instant::now();
    let mut image = Vec::new();
    for index in 0..frames {
        frame(&state, &renderer, &target, index as f32 / 60.0)?;
        if (index + 1) % SYNC_EVERY == 0 || index + 1 == frames {
            image = target.get_image().await;
        }
    }

    // A frame of one flat color means the step drew nothing on top of its background.
    let (pixels, _) = image.as_chunks::<4>();
    let Some(corner) = pixels.first() else {
        bail!("the target returned no pixels; the GPU may have timed out on the readback");
    };
    let total = pixels.len();
    let drawn = pixels.iter().filter(|pixel| *pixel != corner).count();
    if drawn * 100 < total {
        bail!("only {drawn} of {total} pixels differ from the corner");
    }
    let milliseconds = start.elapsed().as_secs_f64() * 1000.0 / f64::from(frames.max(1));
    println!("{frames} frames, {milliseconds:.2} ms each, {drawn} of {total} pixels drawn");

    if let Ok(path) = std::env::var("FC_TUTORIAL_IMAGE") {
        let [width, height] = CHECK_SIZE;
        image::save_buffer(&path, &image, width, height, ColorType::Rgba8)?;
        println!("saved {path}");
    }
    Ok(())
}
