use anyhow::{Context, Result, bail};
use std::{
    io::{Read, Write},
    path::PathBuf,
    process::Stdio,
};

mod code_blocks;
mod config;
mod format;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    files: Option<Vec<PathBuf>>,
    // this option exists to make it easier to use this formatter inside code editors with other markdown formatters
    #[arg(
        long,
        help = "Markdown formatter to run before formatting markdown files"
    )]
    fmt_bin: Option<PathBuf>,
    #[arg(long, help = "Format data from stdin and output to stdout")]
    stdin: bool,
    #[arg(long, help = "Path for configuration file")]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = config::get_config(args.config)?;

    if let Some(files) = args.files {
        for file in files {
            let mut data = std::fs::read_to_string(&file)?;
            if let Some(fmt_bin) = &args.fmt_bin {
                data = format_markdown(fmt_bin, &data)?;
            }
            let mut buff = Vec::new();
            format::format(&config, &data, &mut buff)?;
            std::fs::write(file, buff)?;
        }
    } else {
        let mut data = String::new();
        std::io::stdin().read_to_string(&mut data)?;
        if let Some(fmt_bin) = &args.fmt_bin {
            data = format_markdown(fmt_bin, &data)?;
        }
        format::format(&config, &data, std::io::stdout())?;
    }

    Ok(())
}

fn format_markdown(fmt_bin: &PathBuf, data: &str) -> Result<String> {
    let mut child = std::process::Command::new(fmt_bin)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .as_mut()
        .context("unable to write to formatter stdin")?
        .write_all(data.as_bytes())?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!("failed to spawn markdown format command");
    };

    Ok(String::from_utf8(output.stdout)?)
}
