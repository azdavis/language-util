use std::fs::OpenOptions;
use std::io::{Result, Write as _};
use std::process::{Command, Stdio};

pub(crate) fn rust(name: &std::path::Path, contents: &str) -> Result<()> {
  let prog = Command::new("rustfmt").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn();
  match prog {
    Ok(mut prog) => {
      let mut stdout = prog.stdout.take().unwrap();
      let mut out_file = OpenOptions::new().write(true).create(true).truncate(true).open(name)?;
      prog.stdin.take().unwrap().write_all(contents.as_bytes())?;
      std::io::copy(&mut stdout, &mut out_file)?;
      assert!(prog.wait()?.success());
    }
    Err(_) => {
      // ignore. probably, rustfmt isn't available. just write the file
      // unformatted.
      std::fs::write(name, contents)?;
    }
  }
  Ok(())
}
