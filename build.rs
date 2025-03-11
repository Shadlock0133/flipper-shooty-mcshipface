use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use image::{Rgb, RgbImage};

fn main() {
    println!("cargo::rerun-if-changed=assets/");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut icons = File::create(out_dir.join("icons.rs")).unwrap();
    write_mono_image(&out_dir, "assets/icon.png");
    for file in fs::read_dir("assets").unwrap() {
        let file = file.unwrap();
        if !file.file_type().unwrap().is_file() {
            continue;
        }
        let path = file.path();
        write_icon_const(
            &mut icons,
            &out_dir,
            &path.file_stem().unwrap().to_str().unwrap().to_uppercase(),
            path,
        );
    }
}

fn rgb_to_mono(image: &RgbImage) -> Vec<u8> {
    let (width, height) = image.dimensions();
    let row_size = width.div_ceil(8);

    let mut res = vec![0; (row_size * height) as usize + 1];

    for (y, row) in image.rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            res[y * row_size as usize + (x / 8) + 1] |=
                ((pixel == &Rgb([0; 3])) as u8) << (x % 8);
        }
    }

    res
}

#[track_caller]
fn write_mono_image(out_dir: &Path, src: impl AsRef<Path>) {
    let image = image::open(&src).unwrap().into_rgb8();
    fs::write(
        out_dir.join(src.as_ref().with_extension("icon").file_name().unwrap()),
        rgb_to_mono(&image),
    )
    .unwrap();
}

fn write_icon_const(
    output_file: &mut File,
    out_dir: &Path,
    name: &str,
    src: impl AsRef<Path>,
) {
    let image = image::open(&src).unwrap().into_rgb8();
    let (width, height) = image.dimensions();
    fs::write(
        out_dir.join(src.as_ref().with_extension("icon").file_name().unwrap()),
        rgb_to_mono(&image),
    )
    .unwrap();
    writeln!(
        output_file,
        "pub const {name}: sys::Icon = icon!({width}, {height}, \"{}\");",
        src.as_ref().file_stem().unwrap().to_str().unwrap()
    )
    .unwrap();
}
