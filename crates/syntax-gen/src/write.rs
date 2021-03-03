use std::fs::OpenOptions;
use std::io::{Result, Write as _};
use std::process::{Command, Stdio};

pub(crate) fn write_rust_file(name: &str, contents: &str) -> Result<()> {
  let mut proc = Command::new("rustfmt")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
  proc.stdin.take().unwrap().write_all(contents.as_bytes())?;
  assert!(proc.wait()?.success());
  let mut stdout = proc.stdout.take().unwrap();
  let mut out_file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(name)?;
  std::io::copy(&mut stdout, &mut out_file)?;
  Ok(())
}
