use minifb::{Window, WindowOptions, Key};

fn main() {
    let width = 800;
    let height = 600;

    // Example: a 2D array of pixels (RGB, 0xRRGGBB)
    let pixels_2d: [[u32; 800]; 600] = [[0x00FF00; 800]; 600]; // Green image

    // Flatten the 2D array into a 1D Vec<u32> for minifb
    let mut buffer: Vec<u32> = pixels_2d.iter().flat_map(|row| row.iter().copied()).collect();

    let mut window = Window::new(
        "Pixel Display - minifb",
        width,
        height,
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}