// use the same distance parameter from brakedown
pub const TARGET_DISTANCE: f64 = 0.07;
pub const DISTANCE_THRESHOLD: i32 = ((1.0 / TARGET_DISTANCE) as i32) - 1;
pub const RS_RATE: i32 = 2;
pub const ALPHA: f64 = 0.238;
pub const BETA: f64 = 0.1205;
pub const R: f64 = 1.72;
pub const CN: u64 = 10;
pub const DN: u64 = 20;
pub const COLUMN_SIZE: u64 = 128;
