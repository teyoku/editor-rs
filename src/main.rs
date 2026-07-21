mod buffer;
mod cursor;
mod editor;
mod syntax;
mod terminal;

use buffer::Buffer;
use editor::Editor;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let buffer = match &args[1..].get(0) {
        Some(path) => Buffer::from_file(path),
        None => Buffer::new(),
    };
    let mut editor = Editor::new(buffer);

    editor.run()?;

    Ok(())
}
