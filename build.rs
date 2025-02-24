fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("daniel.ico"); // Make sure daniel.ico is in your project directory or provide a proper path.
        res.compile().unwrap();
    }
}
