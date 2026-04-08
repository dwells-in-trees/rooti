fn main() {
    slint_build::compile_with_config(
        "ui/tree.slint",
        slint_build::CompilerConfiguration::new()
            .with_style("fluent-dark".into()),
    ).unwrap();

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}