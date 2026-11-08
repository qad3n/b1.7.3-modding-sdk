use anyhow::{Result, bail};
use mod_protocol::identifier::isValidModId;
use std::path::{Path, PathBuf};

fn write(path: &Path, text: &str) -> Result<()>
{
    if let Some(parent) = path.parent()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

// the sdk dependency comes from B173_SDK when set, a path to the sdk/b173_api folder
fn sdkDependency() -> String
{
    match std::env::var("B173_SDK")
    {
        Ok(path) => format!("b173_api = {{ path = \"{path}\" }}"),
        Err(_) => "b173_api = \"1\"".to_owned(),
    }
}

pub fn create(parent: &Path, id: &str) -> Result<()>
{
    if !isValidModId(id)
    {
        bail!("mod id '{id}' must match [a-z0-9_]{{2,64}}");
    }
    let root: PathBuf = parent.join(id);
    if root.exists()
    {
        bail!("{} already exists", root.display());
    }
    write(
        &root.join("Cargo.toml"),
        &format!(
            "[package]\nname = \"{id}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\n{}\nserde = {{ version = \"1\", default-features = false, features = [\"alloc\", \"derive\"] }}\n\n[profile.release]\nopt-level = \"s\"\nlto = true\npanic = \"abort\"\n",
            sdkDependency()
        ),
    )?;
    write(
        &root.join("mod.toml"),
        &format!(
            "id = \"{id}\"\nversion = \"0.1.0\"\nname = \"{id}\"\napi = \"1.1\"\nside = \"both\"\n\n[network]\nversion = \"1\"\n\n[depends]\nloader = \">=0.1\"\n"
        ),
    )?;
    write(
        &root.join("src/lib.rs"),
        "use b173_api::prelude::*;\n\n#[mod_main]\nfn register(ctx: &mut RegisterCtx) -> Result<()>\n{\n    ctx.events().on::<Setup>(Priority::Normal, |_| log(\"hello from the mod\"))?;\n    Ok(())\n}\n",
    )?;
    write(
        &root.join(format!("data/{id}/blocks/example_block.toml")),
        "template = \"minecraft:stone\"\nhardness = 1.5\ntexture = \"minecraft:terrain/1\"\nname = \"Example Block\"\n",
    )?;
    write(
        &root.join(format!("assets/{id}/lang/en_US.lang")),
        &format!("tile.{id}.example_block.name=Example Block\n"),
    )?;
    println!("created {}", root.display());
    Ok(())
}
