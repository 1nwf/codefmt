use std::{io::Read, path::PathBuf};

mod code_blocks;
mod config;
mod format;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    files: Option<Vec<PathBuf>>,
    #[arg(long)]
    fmt_bin: Option<PathBuf>,
    #[arg(long)]
    stdin: bool,
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() {
    let mut args = Args::parse();
    let config = config::get_config(args.config);

    if args.files.is_none() {
        args.stdin = true;
    }

    if args.stdin {
        let mut data = String::new();
        std::io::stdin().read_to_string(&mut data).unwrap();
        format::format(&config, &data, std::io::stdout()).unwrap();
    } else {
        let files = args.files.unwrap();
        for file in files {
            let data = std::fs::read_to_string(&file).unwrap();
            let mut buff = Vec::new();
            format::format(&config, &data, &mut buff).unwrap();
            std::fs::write(file, buff).unwrap();
        }
    }
}
