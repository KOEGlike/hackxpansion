fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/neon_beat.slint", config)
        .expect("failed to compile neon beat UI");

    println!("cargo:rerun-if-changed=ui/neon_beat.slint");
    println!("cargo:rerun-if-changed=../../xpanse/ui/fonts/ShareTechMono-Regular.ttf");
}
