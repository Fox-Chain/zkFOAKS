#[cfg(test)]
mod test {
  use crate::LinearPC;
  use prime_field::FieldElement;
  use std::env;
  use std::fs::File;
  use std::io::{BufRead, BufReader};
  use std::path::Path;

  #[test]
  fn test_expander_init() {
    const N: usize = 16384;
    let mut linear_pc = LinearPC::init();
    assert_eq!(220, unsafe {
      linear_pc.lce_ctx.expander_init(N / 128, None)
    });
  }

  #[test]
  fn test_encode() {
    // Depends on C and D x-x
    let mut linear_pc = LinearPC::init();
    let mut root = env::current_dir().unwrap();
    root.pop();
    let src = parse_field_element_vec(root.join("encode_src.txt").as_path());
    let mut encoded_codeword = parse_field_element_vec(root.join("encoded_codeword.txt").as_path());

    println!(
      "src with {} and encoded with {}",
      src.len(),
      encoded_codeword.len()
    );

    assert_eq!(
      220,
      linear_pc.lce_ctx.encode(src, &mut encoded_codeword, 128,)
    );
  }

  fn parse_field_element_vec(file_name: &Path) -> Vec<FieldElement> {
    let file = File::open(file_name).expect("Failed to open the file");

    // Create a BufReader to read the file line by line
    let reader = BufReader::new(file);
    let mut field_elements = vec![];

    // Iterate over each line in the file
    for line in reader.lines() {
      // Unwrap the line or handle any errors that may occur
      match line {
        Ok(line) => {
          // Process the line here
          let mut props = line.split_whitespace();
          field_elements.push(FieldElement::new(
            props.next().unwrap().parse::<u64>().unwrap(),
            props.next().unwrap().parse::<u64>().unwrap(),
          ));
        }
        Err(err) => {
          // Handle the error
          eprintln!("Error reading line: {}", err);
        }
      }
    }

    field_elements
  }
}
