// use the same distance parameter from brakedown
pub const TARGET_DISTANCE: f32 = 0.07;
//pub const DISTANCE_THRESHOLD: i32 = ((1.0 / TARGET_DISTANCE) as i32) - 1;
pub const DISTANCE_THRESHOLD: usize = 13;
//pub const RS_RATE: usize = 2; //never used
pub const ALPHA: f64 = 0.238;
pub const BETA: f64 = 0.1205;
pub const R: f64 = 1.72;
pub const CN: usize = 10;
pub const DN: usize = 20;
pub const COLUMN_SIZE: usize = 128;
