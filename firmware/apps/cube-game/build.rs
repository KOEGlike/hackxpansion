fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/cube_game.slint", config)
        .expect("failed to compile cube game UI");

    println!("cargo:rerun-if-changed=ui/cube_game.slint");
}
