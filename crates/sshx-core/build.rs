use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("sshx.bin");
    tonic_build::configure()
        .file_descriptor_set_path(descriptor_path)
        .bytes(["."])
        .compile_protos(&["proto/sshx.proto"], &["proto/"])?;
    Ok(())
}
