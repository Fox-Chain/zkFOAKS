#[derive(Debug)]
pub struct Paths {
  pub file_path: String,
  pub meta_path: String,
}

impl Paths {
  pub fn build(mut args: impl Iterator<Item = String>) -> Result<Paths, &'static str> {
    args.next();
    args.next();

    let file_path = match args.next() {
      Some(arg) => arg,
      None => return Err("Didn't get a circuit file path"),
    };

    let meta_path = match args.next() {
      Some(arg) => arg,
      None => return Err("Didn't get a meta file path"),
    };

    Ok(Paths {
      file_path,
      meta_path,
    })
  }
}
