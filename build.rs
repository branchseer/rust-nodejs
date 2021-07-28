use sha1::{Digest, Sha1};
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use strum::ToString;

const NODE_VERSION: &str = "v16.5.0";
#[derive(Debug, Eq, PartialEq, Copy, Clone, ToString)]
#[strum(serialize_all = "camelCase")]
enum TargetOS {
    Darwin,
    Win32,
    Linux,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, ToString)]
#[strum(serialize_all = "camelCase")]
enum TargetArch {
    X64,
    X86,
}

#[derive(Debug, Copy, Clone)]
struct Config {
    os: TargetOS,
    arch: TargetArch,
    no_intl: bool,
}

impl Config {
    fn zip_name(&self) -> String {
        format!(
            "libnode-{}-{}-{}{}.zip",
            NODE_VERSION,
            self.os.to_string(),
            self.arch.to_string(),
            if self.no_intl { "-nointl" } else { "" }
        )
    }
    fn url(&self) -> String {
        format!(
            "https://github.com/patr0nus/libnode/releases/download/{}/{}",
            NODE_VERSION,
            self.zip_name()
        )
    }
}

fn get_lib_name(path: &Path, os: Option<TargetOS>) -> Option<&str> {
    if os == Some(TargetOS::Win32) {
        if path.extension()? != OsStr::new("lib") {
            return None;
        }
        path.file_stem()?.to_str()
    } else {
        if path.extension()? != OsStr::new("a") {
            return None;
        }
        let filename = path.file_stem()?.to_str()?;
        filename.strip_prefix("lib")
    }
}

fn verify_sha1_of_file(path: &Path, expected_hex: &str) -> anyhow::Result<()> {
    let mut file = File::open(path)?;
    let mut sha1 = Sha1::default();
    std::io::copy(&mut file, &mut sha1)?;
    let actual_hex = hex::encode(sha1.finalize().as_slice());
    anyhow::ensure!(
        actual_hex == expected_hex,
        "{:?}: sha1 does not match (actual: {}, expected: {})",
        path,
        actual_hex,
        expected_hex
    );
    Ok(())
}

fn get_sha1_of_filename(filename: &str) -> Option<&'static str> {
    for line in include_str!("checksums.sha1").split('\n') {
        let mut line_component_iter = line.trim().split(' ');
        let sha1 = line_component_iter.next()?.trim();
        let fname = line_component_iter.next()?.strip_prefix('*')?;
        if fname == filename {
            return Some(sha1);
        }
    }
    None
}

fn download(url: &str, path: &Path) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let _ = attohttpc::get(url).send()?.write_to(file)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Make docs.rs build pass
    if env::var_os("DOCS_RS").is_some() {
        return Ok(());
    }
    let os = match env::var("CARGO_CFG_TARGET_OS")?.as_str() {
        "macos" => Ok(TargetOS::Darwin),
        "windows" => Ok(TargetOS::Win32),
        "linux" => Ok(TargetOS::Linux),
        other => Err(other.to_string()),
    };
    println!("cargo:rerun-if-env-changed=LIBNODE_LIB_PATH");
    let lib_path = if let Ok(lib_path_from_env) = env::var("LIBNODE_LIB_PATH") {
        println!("cargo:rerun-if-changed={}", lib_path_from_env);
        lib_path_from_env
    } else {
        let os = os
            .clone()
            .map_err(|other| anyhow::anyhow!("Unsupported target arch: {}", other))?;
        let config = Config {
            os,
            arch: match env::var("CARGO_CFG_TARGET_ARCH")?.as_str() {
                "x86" => TargetArch::X86,
                "x86_64" => TargetArch::X64,
                other => anyhow::bail!("Unsupported target arch: {}", other),
            },
            no_intl: env::var("CARGO_FEATURE_NO_INTL").is_ok(),
        };
        let sha1 = if let Some(sha1) = get_sha1_of_filename(config.zip_name().as_str()) {
            sha1
        } else {
            anyhow::bail!("Unsupported config: {:?}", config)
        };
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        let libnode_zip = out_dir.join(config.zip_name());

        if verify_sha1_of_file(libnode_zip.as_path(), sha1).is_err() {
            let url = config.url();
            println!("Downloading {}", url);
            download(url.as_str(), libnode_zip.as_path())?;
            println!("Verifying {:?}", libnode_zip.as_path());
            verify_sha1_of_file(libnode_zip.as_path(), sha1)?;
        }

        let libnode_extracted = out_dir.join("libnode_extracted");
        let _ = std::fs::remove_dir_all(libnode_extracted.as_path());
        println!("Extracting to {:?}", libnode_extracted);
        zip_extract::extract(File::open(libnode_zip)?, &libnode_extracted, true)?;
        libnode_extracted.join("lib").to_str().unwrap().to_string()
    };

    println!("cargo:rustc-link-search=native={}", lib_path);
    for file in std::fs::read_dir(lib_path)? {
        let file = file?;
        if !file.file_type()?.is_file() {
            continue;
        }
        let path = file.path();
        let lib_name = match get_lib_name(path.as_path(), os.clone().ok()) {
            Some(lib_name) => lib_name,
            None => continue,
        };
        println!("cargo:rustc-link-lib=static={}", lib_name);
    }

    let os_specific_libs = match os {
        Ok(TargetOS::Darwin) => vec!["c++"],
        Ok(TargetOS::Linux) => vec!["stdc++"],
        Ok(TargetOS::Win32) => vec!["dbghelp", "winmm", "iphlpapi", "psapi", "crypt32", "user32"],
        Err(_) => vec![],
    };

    for lib_name in os_specific_libs {
        println!("cargo:rustc-link-lib={}", lib_name);
    }

    Ok(())
}
