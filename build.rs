use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

extern crate grass;

#[cfg(debug_assertions)]
fn styles() -> String {
    let format = grass::Options::default().style(grass::OutputStyle::Expanded);
    grass::from_path("public/style.scss", &format).unwrap()
}

#[cfg(not(debug_assertions))]
fn styles() -> String {
    grass::include!("public/style.scss").to_string()
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("style.css");
    let mut f = File::create(dest_path).unwrap();
    f.write_all(styles().as_bytes()).unwrap();

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let filename = "gen/apple/Sources/mobile-demo/bindings/bindings.h";

    let _build = cbindgen::Builder::new()
        .with_crate(crate_dir)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(filename);
}
