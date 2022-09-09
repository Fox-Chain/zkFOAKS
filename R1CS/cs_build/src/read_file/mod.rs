use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Trace {
    pub step: Number,
    pub mOp: bool,
    pub mWr: bool,
    pub addr: BigInt,
    pub value: Vec<Number>,
}
