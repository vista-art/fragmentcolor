use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use std::time::{Duration, Instant};
use std::{io::Write, time};

use crossterm::{
    ExecutableCommand, event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use fragmentcolor::{Renderer, Shader, Target};

pub enum ColorMode {
    TrueColor,
    Xterm256,
}

impl ColorMode {
    pub fn from_env() -> Self {
        match std::env::var("FC_TUI_COLOR").ok().as_deref() {
            Some("256") => Self::Xterm256,
            _ => Self::TrueColor,
        }
    }
}

pub struct TerminalRenderer<W: Write> {
    term: Terminal<CrosstermBackend<W>>,
    renderer: Renderer,
    target: fragmentcolor::TextureTarget,
    cols: u16,
    rows: u16,
    color_mode: ColorMode,
}

impl<W: Write> TerminalRenderer<W> {
    pub fn build(term: Terminal<CrosstermBackend<W>>) -> anyhow::Result<Self> {
        let sz = term.size()?;
        let cols = sz.width;
        let rows = sz.height;

        let renderer = Renderer::new();
        let target = pollster::block_on(
            // texture is double height for half-block mapping
            renderer.create_texture_target([cols as u32, (rows as u32) * 2]),
        )?;

        Ok(Self {
            term,
            renderer,
            target,
            cols,
            rows,
            color_mode: ColorMode::from_env(),
        })
    }

    pub fn handle_resize(&mut self) -> anyhow::Result<()> {
        let sz = self.term.size()?;
        if sz.width != self.cols || sz.height != self.rows {
            self.cols = sz.width;
            self.rows = sz.height;
            self.target
                .resize([self.cols as u32, (self.rows as u32) * 2]);
        }
        Ok(())
    }

    pub fn draw_shader(&mut self, shader: &Shader) -> anyhow::Result<()> {
        self.renderer.render(shader, &self.target)?;
        let rgba = self.target.get_image();
        let size = self.target.size();
        let widget = match self.color_mode {
            ColorMode::TrueColor => rgba_to_half_block_cells_true(&rgba, size.width, size.height),
            ColorMode::Xterm256 => rgba_to_half_block_cells_256(&rgba, size.width, size.height),
        };
        self.term.draw(|f| {
            let area = f.area();
            f.render_widget(widget, area);
        })?;
        Ok(())
    }
}

// char, foreground rgb, background rgb
pub type Grid = Vec<(char, (u8, u8, u8), (u8, u8, u8))>;
pub struct CellsWidget {
    cols: u16,
    rows: u16,
    grid: Grid,
}

impl Widget for CellsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = self.cols.min(area.width);
        let height = self.rows.min(area.height);
        for y in 0..height {
            for x in 0..width {
                let idx = (y as usize) * (self.cols as usize) + (x as usize);
                let (ch, fg, bg) = self.grid[idx];
                let style = Style::default()
                    .fg(Color::Rgb(fg.0, fg.1, fg.2))
                    .bg(Color::Rgb(bg.0, bg.1, bg.2));
                if let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) {
                    cell.set_char(ch).set_style(style);
                }
            }
        }
    }
}

fn rgba_to_half_block_cells_true(rgba: &[u8], w: u32, h: u32) -> CellsWidget {
    rgba_to_half_block_cells(rgba, w, h, |r, g, b| (r, g, b))
}

fn rgba_to_half_block_cells_256(rgba: &[u8], w: u32, h: u32) -> CellsWidget {
    rgba_to_half_block_cells(rgba, w, h, xterm256_quantize)
}

fn rgba_to_half_block_cells<F>(rgba: &[u8], w: u32, h: u32, mut map: F) -> CellsWidget
where
    F: FnMut(u8, u8, u8) -> (u8, u8, u8),
{
    let cols = w as u16;
    let rows = (h / 2) as u16;
    let mut grid = Vec::with_capacity((cols as usize) * (rows as usize));
    for cy in 0..rows {
        let y_top = (cy as u32) * 2;
        let y_bot = y_top + 1;
        for cx in 0..cols {
            let x = cx as u32;
            let i_top = ((y_top * w + x) * 4) as usize;
            let i_bot = ((y_bot.min(h - 1) * w + x) * 4) as usize;
            let top = map(rgba[i_top], rgba[i_top + 1], rgba[i_top + 2]);
            let bot = map(rgba[i_bot], rgba[i_bot + 1], rgba[i_bot + 2]);
            grid.push(('▀', top, bot));
        }
    }
    CellsWidget { cols, rows, grid }
}

fn xterm256_quantize(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    // Very simple cube quantization to the 6x6x6 color cube, expand back to 0..255
    let q = |c: u8| -> u8 { (c as u16 * 5 / 255) as u8 };
    let e = |q: u8| -> u8 { (q as u16 * 255 / 5) as u8 };
    (e(q(r)), e(q(g)), e(q(b)))
}

pub fn run_triangle_demo() -> anyhow::Result<()> {
    // Setup terminal
    std::io::stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let term = Terminal::new(backend)?;
    let mut view = TerminalRenderer::build(term)?;

    let tri_src: &str = include_str!("../examples/shaders/hello_triangle.wgsl");
    let shader = {
        let s = Shader::new(tri_src)?;
        s.set("color", [1.0, 0.2, 0.8, 1.0])?;
        s
    };

    let mut last = Instant::now();
    loop {
        // input
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                event::Event::Key(k) => {
                    if let event::KeyCode::Char('q') | event::KeyCode::Esc = k.code {
                        cleanup_terminal()?;
                        return Ok(());
                    }
                }
                event::Event::Resize(_, _) => {
                    view.handle_resize()?;
                }
                _ => {}
            }
        }

        view.draw_shader(&shader)?;

        // ~30fps
        let now = Instant::now();
        let dt = now - last;
        last = now;
        if dt < time::Duration::from_millis(33) {
            std::thread::sleep(time::Duration::from_millis(33) - dt);
        }
    }
}

pub fn cleanup_terminal() -> anyhow::Result<()> {
    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
