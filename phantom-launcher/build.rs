fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("phantom.ico");
        res.compile().unwrap();
    }
}
