use prime_field::FieldElement;
use std::vec::Vec;

// TODO: check initialization

#[derive(Default, Clone)]
pub struct Graph {
  pub degree: i32,
  pub neighbor: Vec<Vec<u64>>,
  pub r_neighbor: Vec<Vec<u64>>,
  pub weight: Vec<Vec<FieldElement>>,
  pub r_weight: Vec<Vec<FieldElement>>,
  pub l: u64,
  pub r: u64,
}
// TODO this can be something like
// this https://crates.io/crates/lazy_static or this https://github.com/matklad/once_cell

//#[derive(Default)]
// pub struct ExpanderContext {
//   pub c: Vec<Graph>,
//   pub d: Vec<Graph>,
// }

// impl ExpanderContext {
//   pub fn default() -> Self {
//     Self {
//       c: vec![Graph::default(); 100],
//       d: vec![Graph::default(); 100],
//     }
//   }
