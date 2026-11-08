mod check;
mod package;
mod scaffold;

use anyhow::{Context, Result, bail};
use std::path::PathBuf;

const USAGE: &str = "usage: cargo b173 <new <id> | build | check | run --client|--server> [--path <mod dir>]";

struct Args
{
    m_command: String,
    m_rest: Vec<String>,
    m_path: PathBuf,
}

fn parseArgs() -> Result<Args>
{
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // cargo passes the subcommand name first when run as cargo b173
    if args.first().is_some_and(|first| first == "b173")
    {
        args.remove(0);
    }
    let mut path: PathBuf = std::env::current_dir().context("no working directory")?;
    if let Some(index) = args.iter().position(|arg| arg == "--path")
    {
        let value: String = args.get(index + 1).cloned().context("--path needs a directory")?;
        path = PathBuf::from(value);
        args.drain(index..=index + 1);
    }
    if args.is_empty()
    {
        bail!(USAGE);
    }
    let command: String = args.remove(0);
    Ok(Args { m_command: command, m_rest: args, m_path: path })
}

fn main() -> Result<()>
{
    let args: Args = parseArgs()?;
    match args.m_command.as_str()
    {
        "new" =>
        {
            let id: &str = args.m_rest.first().context("cargo b173 new <id>")?;
            scaffold::create(&args.m_path, id)
        }
        "check" =>
        {
            let problems: Vec<String> = check::run(&args.m_path)?;
            for problem in &problems
            {
                eprintln!("{problem}");
            }
            if !problems.is_empty()
            {
                bail!("{} problems", problems.len());
            }
            println!("mod is valid");
            Ok(())
        }
        "build" =>
        {
            let file: PathBuf = package::build(&args.m_path)?;
            println!("{}", file.display());
            Ok(())
        }
        "run" =>
        {
            let server: bool = match args.m_rest.first().map(String::as_str)
            {
                Some("--client") | None => false,
                Some("--server") => true,
                Some(other) => bail!("unknown run target {other}, use --client or --server"),
            };
            package::run(&args.m_path, server)
        }
        other => bail!("unknown command {other}\n{USAGE}"),
    }
}
