//! The program around each step: it opens a window, calls the step's `setup` once,
//! and calls its `frame` on every redraw with the seconds since the window opened.
//!
//! With `FC_TUTORIAL_FRAMES=n` set, it renders n frames into a 256 by 256 texture,
//! fails if nothing reached it, and exits. `FC_TUTORIAL_IMAGE=path` saves the last frame
//! as a PNG. Together they run a step without a display, the way a reviewer checks it.

use anyhow::{Result, bail};
use fragmentcolor::{App, FragmentColorError, RenderTarget, Renderer, SetupResult, Target};
use image::ColorType;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

/// A step's `frame` with its target type fixed to the one `App` hands out.
pub type Frame<State> = fn(&State, &Renderer, &RenderTarget, f32) -> Result<(), FragmentColorError>;

const STATE: &str = "state";
const CHECK_SIZE: u32 = 256;

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
    if let Some(Err(error)) = drawn {
        eprintln!("frame failed: {error}");
        std::process::exit(1);
    }
}

async fn check<State, Setup>(setup: Setup, frame: Frame<State>, frames: u32) -> Result<()>
where
    Setup: AsyncFnOnce(&Renderer) -> Result<State, FragmentColorError>,
{
    let renderer = Renderer::new();
    let texture = renderer
        .create_texture_target([CHECK_SIZE, CHECK_SIZE])
        .await?;
    let target = RenderTarget::from(texture);
    let state = setup(&renderer).await?;
    for index in 0..frames {
        frame(&state, &renderer, &target, index as f32 / 60.0)?;
    }

    // A pass clears to transparent black, so any other pixel is one the step drew.
    let image = target.get_image().await;
    let (pixels, _) = image.as_chunks::<4>();
    let total = pixels.len();
    let drawn = pixels.iter().filter(|pixel| **pixel != [0; 4]).count();
    if drawn * 100 < total {
        bail!("only {drawn} of {total} pixels were drawn");
    }
    println!("{frames} frames rendered, {drawn} of {total} pixels drawn");

    if let Ok(path) = std::env::var("FC_TUTORIAL_IMAGE") {
        image::save_buffer(&path, &image, CHECK_SIZE, CHECK_SIZE, ColorType::Rgba8)?;
        println!("saved {path}");
    }
    Ok(())
}
