pub mod errors;
pub mod cursor;
pub mod byte_source;
pub mod view;
pub mod specs;
pub mod macros;
pub mod file;
pub mod trys;

pub use byte_source::ByteSource;
pub use cursor::Cursor;
pub use view::View;
pub use specs::Spec;
use file::MmapByteSource;

pub fn default_main<S: Spec<Params = ()>>() {
    use std::io::{self, Write};

    print!("Enter file path: ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("Failed to read stdin");
    let path = line.trim();

    let source = MmapByteSource::from_path(path).expect("Failed to open/map file");

    let (parsed, consumed) = S::read_all(&source, ())
        .expect("Parse failed");

    println!("Parsed: {:?}", parsed);
    println!("Consumed {} bytes", consumed);
}