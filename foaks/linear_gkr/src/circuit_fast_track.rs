//use std::sync::{Arc, Mutex};
//use std::thread;

use prime_field::FieldElement;

#[derive(Debug, Clone, Default)]
pub struct Gate {
  pub ty: usize,
  pub u: usize,
  pub v: usize,
  pub src: Vec<usize>,
  pub weight: Vec<FieldElement>,
  pub parameter_length: usize,
}

impl Gate {
  pub fn new() -> Self {
    Self {
      ty: 2,
      ..Default::default()
    }
  }

  pub fn from_params(ty: usize, u: usize, v: usize) -> Self {
    Self {
      ty,
      u,
      v,
      ..Default::default()
    }
  }
}

#[derive(Default, Debug, Clone)]
pub struct Layer {
  pub src_expander_c_mempool: Vec<usize>,
  pub src_expander_d_mempool: Vec<usize>,
  pub weight_expander_c_mempool: Vec<FieldElement>,
  pub weight_expander_d_mempool: Vec<FieldElement>,
  pub gates: Vec<Gate>,
  pub bit_length: usize,
  pub is_parallel: bool,
  pub block_size: usize,
  pub log_block_size: usize,
  pub repeat_num: usize,
  pub log_repeat_num: usize,
}

#[derive(Default, Debug, Clone)]
pub struct LayeredCircuit {
  pub circuit: Vec<Layer>,
  pub total_depth: usize,
  pub inputs: Vec<FieldElement>,
}
//advice to implement parallelism

//impl LayeredCircuit {
// pub fn process_gates_in_parallel(&mut self) {
//  let gates = Arc::new(Mutex::new(self.circuit[0].gates.clone()));
// let mut handles = vec![];

// for gate in gates.lock().unwrap().iter_mut() {
//  let gate_clone = gate.clone();
// let handle = thread::spawn(move || {

//gate_clone.ty += 1;
//});
//handles.push(handle);
// }

//for handle in handles {
//handle.join().unwrap();
//}
// }
//}
