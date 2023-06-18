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
    grass::include!("../public/style.scss").to_string()
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("style.css");
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(styles().as_bytes()).unwrap();
}