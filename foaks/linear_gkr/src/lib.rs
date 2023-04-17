pub mod circuit_fast_track;
pub mod config;
pub mod polynomial;
pub mod prover;
pub mod verifier;

#[cfg(test)]
mod test {
  use crate::config::Paths;

  #[test]
  fn print_paths() {
    let circuit = "/workspaces/zkFOAKS/foaks/linear_gkr/mat_16_circuit.txt";
    let meta = "/workspaces/zkFOAKS/foaks/linear_gkr/mat_16_meta.txt";

    let envs = vec!["", "", circuit, meta]
      .iter()
      .map(|s| String::from(*s))
      .collect::<Vec<String>>();

    let paths = Paths::build(envs.into_iter()).unwrap();
    assert_eq!(paths.file_path, circuit);
    assert_eq!(paths.meta_path, meta);
  }
}
