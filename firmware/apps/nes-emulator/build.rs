use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=NES_ROM");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let generated = out_dir.join("rom.rs");
    let source = match env::var_os("NES_ROM") {
        Some(path) => {
            let path = PathBuf::from(path)
                .canonicalize()
                .expect("NES_ROM must point to a readable iNES ROM");
            println!("cargo:rerun-if-changed={}", path.display());
            format!("pub static ROM: &[u8] = include_bytes!({path:?});\n")
        }
        None => {
            println!(
                "cargo:warning=NES emulator disabled: set NES_ROM=/path/to/game.nes when building"
            );
            "pub static ROM: &[u8] = &[];\n".to_owned()
        }
    };

    fs::write(generated, source).expect("failed to generate ROM include");
}
