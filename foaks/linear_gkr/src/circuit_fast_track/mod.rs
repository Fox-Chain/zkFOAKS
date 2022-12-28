use std::collections::HashMap;

use prime_field::FieldElement;
#[derive(Debug)]

pub struct Gate {
    pub ty: i32,
    pub u: usize,
    pub v: usize,
    pub src: Vec<i32>,
    pub weight: Vec<FieldElement>,
    pub parameter_length: usize,
}

impl Default for Gate {
    fn default() -> Self {
        Self {
            ty: 2,
            u: 0,
            v: 0,
            src: vec![],
            weight: vec![],
            parameter_length: 0,
        }
    }
}

impl Gate {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_params(ty: i32, u: usize, v: usize) -> Self {
        Self {
            ty,
            u,
            v,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct Layer {
    src_expander_c_mempool: Vec<i32>,
    src_expander_d_mempool: Vec<i32>,
    weight_expander_c_mempool: Vec<FieldElement>,
    weight_expander_d_mempool: Vec<FieldElement>,
    pub gates: Vec<Gate>,
    pub bit_length: usize,
    u_gates: HashMap<i32, Vec<(i32, (i32, i32))>>,
    v_gates: HashMap<i32, Vec<(i32, (i32, i32))>>,
    is_parallel: bool,
    block_size: usize,
    log_block_size: usize,
    repeat_num: usize,
    log_repeat_num: usize,
}

impl Layer {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default, Debug)]
pub struct LayeredCircuit {
    pub circuit: Vec<Layer>,
    pub total_depth: usize,
    pub nputs: Vec<FieldElement>,
}

impl LayeredCircuit {
    pub fn new() -> Self {
        Default::default()
    }
}
