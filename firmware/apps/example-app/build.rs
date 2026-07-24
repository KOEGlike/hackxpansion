fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/button_logger.slint", config)
        .expect("failed to compile button logger UI");

    println!("cargo:rerun-if-changed=ui/button_logger.slint");
}
