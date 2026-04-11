use resvg::{tiny_skia, usvg};

fn main() {
    let svg_path = concat!(env!("CARGO_MANIFEST_DIR"), "/LiteRequestIcon.svg");
    println!("cargo:rerun-if-changed={svg_path}");

    let svg_data = std::fs::read(svg_path).expect("Failed to read LiteRequestIcon.svg");

    let mut opts = usvg::Options::default();
    opts.resources_dir = Some(
        std::path::Path::new(svg_path)
            .parent()
            .unwrap()
            .to_path_buf(),
    );

    let tree = usvg::Tree::from_data(&svg_data, &opts).expect("Failed to parse SVG");

    let size = 256u32;
    let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("Failed to create pixmap");
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let png_data = pixmap.encode_png().expect("Failed to encode PNG");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::Path::new(&out_dir).join("app_icon.png");
    std::fs::write(out_path, png_data).expect("Failed to write icon PNG");
}
