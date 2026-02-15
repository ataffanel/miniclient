fn main() {
    slint_build::compile("ui/main.slint").unwrap();

    #[cfg(windows)]
    windows_resources();
}

#[cfg(windows)]
fn windows_resources() {
    use std::env;
    use std::path::PathBuf;

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ico_path = out_dir.join("app.ico");

    generate_ico("icon.png", &ico_path);

    // Also write a copy into wix/ so that cargo wix can reference it.
    let wix_ico = PathBuf::from("wix/app.ico");
    std::fs::copy(&ico_path, &wix_ico).expect("Failed to copy app.ico to wix/");

    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico_path.to_str().unwrap());
    res.compile().unwrap();
}

#[cfg(windows)]
fn generate_ico(png_path: &str, ico_path: &std::path::Path) {
    use image::imageops::FilterType;

    let img = image::open(png_path).expect("Failed to open icon.png");
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for size in [16u32, 32, 48, 256] {
        let resized = img.resize_exact(size, size, FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let icon_image = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
        icon_dir
            .add_entry(ico::IconDirEntry::encode(&icon_image).unwrap());
    }

    let file = std::fs::File::create(ico_path).unwrap();
    icon_dir.write(file).unwrap();
}
