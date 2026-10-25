/// You can run the examples from the root workspace with
/// ./fc example <name>
mod tui_util;

fn main() -> anyhow::Result<()> {
    tui_util::run_triangle_demo()
}
