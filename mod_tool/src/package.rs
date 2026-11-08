use crate::check;
use anyhow::{Context, Result, bail};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const WASM_TARGET: &str = "wasm32-unknown-unknown";
const PACKED: [&str; 2] = ["data", "assets"];

fn crateName(root: &Path) -> Result<String>
{
    let text: String = std::fs::read_to_string(root.join("Cargo.toml")).context("Cargo.toml")?;
    let manifest: toml::Table = toml::from_str(&text).context("Cargo.toml")?;
    let name: &str = manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .context("Cargo.toml has no package name")?;
    Ok(name.replace('-', "_"))
}

fn targetDir(root: &Path) -> Result<PathBuf>
{
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(root)
        .output()
        .context("cargo metadata")?;
    if !output.status.success()
    {
        bail!("cargo metadata failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).context("cargo metadata output")?;
    Ok(PathBuf::from(metadata["target_directory"].as_str().context("cargo metadata has no target directory")?))
}

fn compile(root: &Path) -> Result<Option<Vec<u8>>>
{
    if !root.join("Cargo.toml").is_file()
    {
        return Ok(None);
    }
    let status = Command::new("cargo")
        .args(["build", "--release", "--target", WASM_TARGET])
        .current_dir(root)
        .status()
        .context("cargo build")?;
    if !status.success()
    {
        bail!("the mod did not compile, is the target installed? rustup target add {WASM_TARGET}");
    }
    let wasm: PathBuf = targetDir(root)?.join(WASM_TARGET).join("release").join(format!("{}.wasm", crateName(root)?));
    Ok(Some(std::fs::read(&wasm).with_context(|| format!("{} is missing", wasm.display()))?))
}

// one zip with mod.toml, mod.wasm when the mod has code, and its data and assets folders
pub fn build(root: &Path) -> Result<PathBuf>
{
    let problems: Vec<String> = check::run(root)?;
    if !problems.is_empty()
    {
        bail!("cargo b173 check found problems:\n{}", problems.join("\n"));
    }
    let head: check::ManifestHead = check::manifest(root)?;
    let wasm: Option<Vec<u8>> = compile(root)?;
    let outDir: PathBuf = root.join("target");
    std::fs::create_dir_all(&outDir)?;
    let output: PathBuf = outDir.join(format!("{}-{}.b173mod", head.id, head.version));
    let mut writer: zip::ZipWriter<std::fs::File> = zip::ZipWriter::new(std::fs::File::create(&output)?);
    let options: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    writer.start_file("mod.toml", options)?;
    writer.write_all(&std::fs::read(root.join("mod.toml"))?)?;
    if let Some(wasm) = wasm
    {
        writer.start_file("mod.wasm", options)?;
        writer.write_all(&wasm)?;
    }
    for folder in PACKED
    {
        for path in check::files(root, folder)
        {
            let name: String = path.strip_prefix(root)?.to_string_lossy().replace('\\', "/");
            writer.start_file(name, options)?;
            writer.write_all(&std::fs::read(&path)?)?;
        }
    }
    writer.finish()?;
    Ok(output)
}

fn gameDir(server: bool, root: &Path) -> PathBuf
{
    let variable: &str = if server
    {
        "B173_SERVER_DIR"
    }
    else
    {
        "B173_GAME_DIR"
    };
    match std::env::var_os(variable)
    {
        Some(dir) => PathBuf::from(dir),
        None => root.join("run").join(if server { "server" } else { "client" }),
    }
}

// builds, copies the file into the game's mods folder and starts the game binary there
pub fn run(root: &Path, server: bool) -> Result<()>
{
    let file: PathBuf = build(root)?;
    let dir: PathBuf = gameDir(server, root);
    let mods: PathBuf = dir.join("mods");
    std::fs::create_dir_all(&mods)?;
    std::fs::copy(&file, mods.join(file.file_name().context("built file has a name")?))?;
    let binary: &str = if server { "beta173_server" } else { "beta173" };
    let program: PathBuf =
        std::env::var_os("B173_BIN_DIR").map_or_else(|| PathBuf::from(binary), |bin| PathBuf::from(bin).join(binary));
    let status = Command::new(&program)
        .current_dir(&dir)
        .status()
        .with_context(|| format!("could not start {}, set B173_BIN_DIR", program.display()))?;
    if !status.success()
    {
        bail!("{binary} exited with {status}");
    }
    Ok(())
}
