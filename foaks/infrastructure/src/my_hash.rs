#[derive(Default)]
pub struct HashDigest {
    h0: i128,
    h1: i128,
}

impl HashDigest {
    pub fn new() -> Self {
        Default::default()
    }
}
