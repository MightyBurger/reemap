const ICON_PATH: &str = "resource/lurk.ico";

fn main() {
    compile_resource();
    extract_runtime_icon();
}

fn compile_resource() {
    let mut res = winres::WindowsResource::new();
    res.set_icon(ICON_PATH);
    res.compile().unwrap();
}

fn extract_runtime_icon() {
    use std::env;
    use std::fs;
    use std::path::Path;

    let icon_file = fs::File::open(ICON_PATH).unwrap();
    let icon_dir = ico::IconDir::read(icon_file).unwrap();
    let out_dir = env::var_os("OUT_DIR").unwrap();

    // 32 x 32 icon raw
    let runtime_icon32_entry = icon_dir
        .entries()
        .into_iter()
        .find(|entry| (entry.width(), entry.height()) == (32, 32))
        .expect("the icon does not contain a 32 x 32 icon");
    let image32 = runtime_icon32_entry.decode().unwrap();
    let rgba32 = image32.rgba_data();

    let icon32_raw_path = Path::new(&out_dir).join("icon32.raw");
    println!(
        "cargo:rustc-env=ICON32_RAW_PATH={}",
        icon32_raw_path.to_str().unwrap()
    );
    fs::write(&icon32_raw_path, rgba32).unwrap();

    // 256 x 256  icon raw
    let runtime_icon256_entry = icon_dir
        .entries()
        .into_iter()
        .find(|entry| (entry.width(), entry.height()) == (256, 256))
        .expect("the icon does not contain a 256 x 256 icon");
    let image256 = runtime_icon256_entry.decode().unwrap();
    let rgba256 = image256.rgba_data();

    let icon256_raw_path = Path::new(&out_dir).join("icon256.raw");
    println!(
        "cargo:rustc-env=ICON256_RAW_PATH={}",
        icon256_raw_path.to_str().unwrap()
    );
    fs::write(&icon256_raw_path, rgba256).unwrap();
}
