//! A task runner using the [xtask spec][1].
//!
//! [1]: https://github.com/matklad/cargo-xtask

use anyhow::{bail, Result};
use pico_args::Arguments;
use std::path::Path;
use xshell::{cmd, pushd};

#[inline]
fn show_help() {
  print!("{}", include_str!("help.txt"));
}

fn finish_args(args: Arguments) -> Result<()> {
  let args = args.finish();
  if !args.is_empty() {
    bail!("unused arguments: {:?}", args);
  }
  Ok(())
}

fn run() -> Result<()> {
  let mut args = Arguments::from_env();
  if args.contains(["-h", "--help"]) {
    show_help();
    return Ok(());
  }
  let subcommand = match args.subcommand()? {
    Some(x) => x,
    None => {
      show_help();
      return Ok(());
    }
  };
  let _d = pushd(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())?;
  match subcommand.as_str() {
    "ci" => {
      finish_args(args)?;
      cmd!("cargo test --no-run").run()?;
      cmd!("cargo fmt -- --check").run()?;
      cmd!("cargo clippy").run()?;
      cmd!("cargo test").run()?;
    }
    s => bail!("unknown subcommand: {}", s),
  }
  Ok(())
}

fn main() {
  match run() {
    Ok(()) => {}
    Err(e) => {
      eprintln!("{}", e);
      std::process::exit(1);
    }
  }
}
