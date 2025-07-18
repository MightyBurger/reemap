fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("resource/lurk.ico");
    res.compile().unwrap();
}
