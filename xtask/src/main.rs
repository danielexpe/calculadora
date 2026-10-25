use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const REQUIRED_FONTS: &[&str] = &[
    "fonts/Inter-Regular.ttf",
    "fonts/Inter-SemiBold.ttf",
    "fonts/JetBrainsMono-Regular.ttf",
    "fonts/JetBrainsMono-Medium.ttf",
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command] if command == "assets" => run_assets(false),
        [command, flag] if command == "assets" && flag == "--check" => run_assets(true),
        _ => {
            eprintln!("usage: cargo xtask assets [--check]");
            ExitCode::from(2)
        }
    }
}

fn run_assets(check: bool) -> ExitCode {
    match assets(check) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn assets(check: bool) -> Result<(), String> {
    let root = project_root();
    let assets_dir = root.join("assets");
    let manifest_path = assets_dir.join("generated").join("manifest.txt");
    let font_dir = assets_dir.join("fonts");

    for required in REQUIRED_FONTS {
        let path = assets_dir.join(required);
        if !path.is_file() {
            return Err(format!("missing required asset: {}", path.display()));
        }
    }

    let files = collect_files(&font_dir)?;
    let manifest = render_manifest(&assets_dir, &files)?;

    if check {
        let current = fs::read_to_string(&manifest_path).map_err(|error| {
            format!(
                "cannot read {}: {error}; run `cargo xtask assets`",
                manifest_path.display()
            )
        })?;
        if current != manifest {
            return Err(format!(
                "{} is out of date; run `cargo xtask assets`",
                manifest_path.display()
            ));
        }
        println!("assets manifest is up to date");
        return Ok(());
    }

    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&manifest_path, manifest)
        .map_err(|error| format!("cannot write {}: {error}", manifest_path.display()))?;
    println!("wrote {}", manifest_path.display());
    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must live under the project root")
        .to_path_buf()
}

fn collect_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_files_into(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_into(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_into(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn render_manifest(assets_dir: &Path, files: &[PathBuf]) -> Result<String, String> {
    let mut rows = BTreeMap::new();
    for path in files {
        let relative = path
            .strip_prefix(assets_dir)
            .map_err(|error| format!("cannot relativize {}: {error}", path.display()))?;
        let relative = relative.to_string_lossy().replace('\\', "/");
        let bytes =
            fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        rows.insert(relative, (fnv1a64(&bytes), bytes.len()));
    }

    let mut manifest = String::from("# calculadora assets manifest v1\n");
    manifest.push_str("# fnv1a64  size  path\n");
    for (path, (hash, size)) in rows {
        manifest.push_str(&format!("{hash:016x}  {size}  {path}\n"));
    }
    Ok(manifest)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::fnv1a64;

    #[test]
    fn fnv1a64_known_vectors() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }
}
