const REQUIRED_THRESHOLD: usize = 14;

#[derive(Debug)]
pub enum MainError {
  BelowThreshold,
  ParseParamsError,
  NoNumberProvided,
}

pub fn parse_number(input: &str) -> Result<usize, MainError> {
  match input.parse::<usize>() {
    Ok(n) if n >= REQUIRED_THRESHOLD => Ok(n),
    Ok(_) => Err(MainError::BelowThreshold),
    Err(_) => Err(MainError::ParseParamsError),
  }
}
