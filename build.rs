fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/active.ico");
        res.set("ProductName", "Caffeine");
        res.set("FileDescription", "Caffeine - Keep Screen Awake");
        res.set("LegalCopyright", "Copyright (C) 2026");
        // winres will automatically find llvm-rc / windres
        if let Err(e) = res.compile() {
            eprintln!("Warning: could not compile Windows resource: {}", e);
        }
    }
}
