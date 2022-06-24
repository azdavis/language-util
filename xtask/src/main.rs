//! A task runner using the [xtask spec][1].
//!
//! [1]: https://github.com/matklad/cargo-xtask

use anyhow::{bail, Result};
use pico_args::Arguments;
use std::path::Path;
use xshell::{cmd, Shell};

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

fn main() -> Result<()> {
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
  let sh = Shell::new()?;
  let _d = sh.push_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap());
  match subcommand.as_str() {
    "ci" => {
      finish_args(args)?;
      cmd!(sh, "cargo test --no-run").run()?;
      cmd!(sh, "cargo fmt -- --check").run()?;
      cmd!(sh, "cargo clippy").run()?;
      cmd!(sh, "cargo test").run()?;
    }
    s => bail!("unknown subcommand: {}", s),
  }
  Ok(())
}
