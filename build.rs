use ring::digest::Digest;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use strum::ToString;

const NODE_VERSION: &str = "v16.9.1";
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
    full_icu: bool,
}

impl Config {
    fn zip_name(&self) -> String {
        format!(
            "libnode-{}-{}-{}{}.zip",
            NODE_VERSION,
            self.os.to_string(),
            self.arch.to_string(),
            if self.full_icu { "" } else { "-small_icu" }
        )
    }
    fn url(&self) -> String {
        format!(
            "https://github.com/patr0nus/rust-nodejs/releases/download/libnode-{}/{}",
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

fn sha256_digest(mut reader: impl io::Read) -> io::Result<Digest> {
    use ring::digest::{Context, SHA256};

    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 8 * 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

fn verify_sha256_of_file(path: &Path, expected_hex: &str) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let sha256 = sha256_digest(file)?;
    let actual_hex = hex::encode(sha256.as_ref());
    anyhow::ensure!(
        actual_hex == expected_hex,
        "{:?}: sha256 does not match (actual: {}, expected: {})",
        path,
        actual_hex,
        expected_hex
    );
    Ok(())
}

fn get_sha256_for_filename(filename: &str) -> Option<&'static str> {
    for line in include_str!("checksums").split('\n') {
        let mut line_component_iter = line.trim().split(' ');
        let sha256 = line_component_iter.next()?.trim();
        let fname = line_component_iter.next()?.strip_prefix('*')?;
        if fname == filename {
            return Some(sha256);
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
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let os = match env::var("CARGO_CFG_TARGET_OS")?.as_str() {
        "macos" => Ok(TargetOS::Darwin),
        "windows" => Ok(TargetOS::Win32),
        "linux" => Ok(TargetOS::Linux),
        other => Err(other.to_string()),
    };
    let arch = match env::var("CARGO_CFG_TARGET_ARCH")?.as_str() {
        "x86" => Ok(TargetArch::X86),
        "x86_64" => Ok(TargetArch::X64),
        other => Err(other.to_string()),
    };
    if let Ok(TargetOS::Win32) = os {
        let target_env = env::var("CARGO_CFG_TARGET_ENV")?;
        if target_env != "msvc" {
            // Can't link to Nodejs under windows-gnu
            anyhow::bail!("Unsupported Environment ABI: {}", target_env)
        }
    }
    println!("cargo:rerun-if-env-changed=LIBNODE_PATH");
    let libnode_path = if let Ok(libnode_path_from_env) = env::var("LIBNODE_PATH") {
        println!("cargo:rerun-if-changed={}", libnode_path_from_env);
        PathBuf::from(libnode_path_from_env)
    } else {
        let config = Config {
            os: match os.clone() {
                Ok(os) => os,
                Err(other) => anyhow::bail!("Unsupported target os: {}", other),
            },
            arch: match arch.clone() {
                Ok(arch) => arch,
                Err(other) => anyhow::bail!("Unsupported target arch: {}", other),
            },
            full_icu: env::var("CARGO_FEATURE_FULL_ICU").is_ok(),
        };
        let sha256 = get_sha256_for_filename(config.zip_name().as_str()).expect(&format!(
            "No sha256 checksum found for filename: {}",
            config.zip_name().as_str()
        ));
        let libnode_zip = out_dir.join(config.zip_name());

        if verify_sha256_of_file(libnode_zip.as_path(), sha256).is_err() {
            let url = config.url();
            println!("Downloading {}", url);
            download(url.as_str(), libnode_zip.as_path())?;
            println!("Verifying {:?}", libnode_zip.as_path());
            verify_sha256_of_file(libnode_zip.as_path(), sha256)?;
        }

        let libnode_extracted = out_dir.join("libnode_extracted");
        let _ = std::fs::remove_dir_all(libnode_extracted.as_path());
        println!("Extracting to {:?}", libnode_extracted);
        zip_extract::extract(File::open(libnode_zip)?, &libnode_extracted, true)?;
        libnode_extracted
    };

    std::fs::copy(libnode_path.join("sys.rs"), out_dir.join("sys.rs"))?;

    let lib_path = libnode_path.join("lib");

    println!(
        "cargo:rustc-link-search=native={}",
        lib_path.to_str().unwrap()
    );
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

    let os_libs = match os {
        Ok(TargetOS::Darwin) => ["c++"].as_ref(),
        Ok(TargetOS::Linux) => ["stdc++"].as_ref(),
        Ok(TargetOS::Win32) => {
            ["dbghelp", "winmm", "iphlpapi", "psapi", "crypt32", "user32"].as_ref()
        }
        Err(_) => [].as_ref(),
    };
    for os_lib_name in os_libs {
        println!("cargo:rustc-link-lib={}", *os_lib_name);
    }
    Ok(())
}
