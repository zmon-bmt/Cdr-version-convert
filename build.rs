fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("cdr_conv_icon.ico");
        res.set("FileDescription", "CDR Version Converter");
        res.set("ProductName", "CDR Version Converter");
        res.set("OriginalFilename", "cdr_converter.exe");
        res.set("LegalCopyright", "Open Source");
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile Windows resource: {}", e);
        }
    }
}
