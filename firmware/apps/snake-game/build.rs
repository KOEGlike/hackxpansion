fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/snake_game.slint", config)
        .expect("failed to compile snake game UI");

    println!("cargo:rerun-if-changed=ui/snake_game.slint");
}
