use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use hex;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

fn compress(binary: Vec<u8>) -> Result<Vec<u8>> {
    let mut writer = GzEncoder::new(Vec::<u8>::with_capacity(binary.len()), Compression::best());
    writer.write_all(&binary)?;
    Ok(writer.finish()?)
}

fn build_alkane(wasm_str: &str, features: Vec<&'static str>) -> Result<()> {
    if features.len() != 0 {
        let _ = Command::new("cargo")
            .env("CARGO_TARGET_DIR", wasm_str)
            .arg("build")
            .arg("--release")
            .arg("--features")
            .arg(features.join(","))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        Ok(())
    } else {
        Command::new("cargo")
            .env("CARGO_TARGET_DIR", wasm_str)
            .arg("build")
            .arg("--release")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=alkanes/");
    println!("cargo:rerun-if-changed=submodules/");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = Path::new(&manifest_dir);

    let wasm_dir = project_root.join("target").join("alkanes");
    fs::create_dir_all(&wasm_dir).unwrap();
    let wasm_str = wasm_dir.to_str().unwrap();

    let write_dir = project_root.join("src").join("tests");
    fs::create_dir_all(write_dir.join("std")).unwrap();

    let alkanes_dir = project_root.join("alkanes");
    let submodules_dir = project_root.join("submodules");

    eprintln!("cargo:warning=Project Root: {}", project_root.display());
    eprintln!("cargo:warning=WASM Dir: {}", wasm_dir.display());
    eprintln!("cargo:warning=Write Dir: {}", write_dir.display());
    eprintln!("cargo:warning=Alkanes Dir: {}", alkanes_dir.display());
    eprintln!("cargo:warning=Submodules Dir: {}", submodules_dir.display());

    let mut all_mods = Vec::new();

    // Process alkanes
    if alkanes_dir.exists() {
        all_mods.extend(process_directory(&alkanes_dir, &wasm_str, &write_dir)?);
    }

    // Process submodules
    if submodules_dir.exists() {
        all_mods.extend(process_directory(&submodules_dir, &wasm_str, &write_dir)?);
    }

    // Write mod.rs
    let mod_content = all_mods.into_iter().fold(String::default(), |r, v| {
        r + "pub mod " + v.as_str() + "_build;\n"
    });
    fs::write(&write_dir.join("std").join("mod.rs"), mod_content).unwrap();

    Ok(())
}

fn process_directory(dir: &Path, wasm_str: &str, write_dir: &Path) -> Result<Vec<String>> {
    let mut built_names = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();

                    if dir_name.starts_with('.') {
                        continue;
                    }

                    let cargo_toml = path.join("Cargo.toml");
                    if !cargo_toml.exists() {
                        continue;
                    }

                    eprintln!("Processing directory: {}", dir_name);

                    match std::env::set_current_dir(&path) {
                        Ok(_) => {
                            if let Err(e) = build_alkane(wasm_str, vec![]) {
                                eprintln!("Failed to build {}: {}", dir_name, e);
                                continue;
                            }

                            let subbed = dir_name.replace("-", "_");

                            let wasm_path = Path::new(&wasm_str)
                                .join("wasm32-unknown-unknown")
                                .join("release")
                                .join(format!("{}.wasm", subbed));

                            if let Ok(f) = fs::read(&wasm_path) {
                                if let Ok(compressed) = compress(f.clone()) {
                                    let _ = fs::write(
                                        &Path::new(&wasm_str)
                                            .join("wasm32-unknown-unknown")
                                            .join("release")
                                            .join(format!("{}.wasm.gz", subbed)),
                                        &compressed,
                                    );
                                }

                                let data = hex::encode(&f);
                                let build_content = format!(
                                    "use hex_lit::hex;\n#[allow(long_running_const_eval)]\npub fn get_bytes() -> Vec<u8> {{ (&hex!(\"{}\")).to_vec() }}",
                                    data
                                );

                                let build_file_path =
                                    write_dir.join("std").join(format!("{}_build.rs", subbed));
                                if let Err(e) = fs::write(&build_file_path, build_content) {
                                    eprintln!("Failed to write build file for {}: {}", dir_name, e);
                                    continue;
                                }

                                eprintln!("Successfully processed: {}", dir_name);
                                built_names.push(subbed);
                            } else {
                                eprintln!("Failed to read wasm file for: {}", dir_name);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to change directory to {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }
    let original_dir = write_dir.parent().unwrap().parent().unwrap();
    let _ = std::env::set_current_dir(&original_dir);
    Ok(built_names)
}
