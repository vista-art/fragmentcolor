/// You can run the examples from the root workspace by using
/// cargo run -p fce --example <name>
mod tui_util;

fn main() -> anyhow::Result<()> {
    tui_util::run_triangle_demo()
}
