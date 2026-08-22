use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let include = protoc_bin_vendored::include_path()?;
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    println!("cargo:rerun-if-changed=../../proto/common.proto");
    println!("cargo:rerun-if-changed=../../proto/burp.proto");
    let mut prost_config = prost_build::Config::new();
    prost_config.out_dir(out_dir);
    prost_config.protoc_executable(protoc);
    tonic_prost_build::configure().compile_with_config(
        prost_config,
        &["../../proto/common.proto", "../../proto/burp.proto"],
        &[
            "../../proto",
            include.to_str().ok_or("invalid protoc include path")?,
        ],
    )?;
    Ok(())
}
