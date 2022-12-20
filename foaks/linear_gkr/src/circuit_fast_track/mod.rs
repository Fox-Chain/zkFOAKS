use std::collections::HashMap;

use prime_field::FieldElement;

pub struct Gate<'a> {
    pub ty: i32,
    pub u: usize,
    pub v: usize,
    src: Option<&'a i32>,
    pub weight: Option<&'a FieldElement>,
    pub parameter_length: usize,
}

impl<'a> Default for Gate<'a> {
    fn default() -> Self {
        Self {
            ty: 2,
            u: 0,
            v: 0,
            src: None,
            weight: None,
            parameter_length: 0,
        }
    }
}

impl<'a> Gate<'a> {
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

#[derive(Default)]
pub struct Layer<'a> {
    src_expander_c_mempool: Vec<i32>,
    src_expander_d_mempool: Vec<i32>,
    weight_expander_c_mempool: Vec<FieldElement>,
    weight_expander_d_mempool: Vec<FieldElement>,
    pub gates: Vec<Gate<'a>>,
    pub bit_length: usize,
    u_gates: HashMap<i32, Vec<(i32, (i32, i32))>>,
    v_gates: HashMap<i32, Vec<(i32, (i32, i32))>>,
    is_parallel: bool,
    block_size: usize,
    log_block_size: usize,
    repeat_num: usize,
    log_repeat_num: usize,
}

impl<'a> Layer<'a> {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Default)]
pub struct LayeredCircuit<'a> {
    pub circuit: Vec<Layer<'a>>,
    pub total_depth: usize,
    pub nputs: Vec<FieldElement>,
}

impl<'a> LayeredCircuit<'a> {
    pub fn new() -> Self {
        Default::default()
    }
}
