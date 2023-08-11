use prime_field::FieldElement;

use crate::{constants::MAX_ORDER_FFT, utility::my_log};

#[derive(Default, Debug, Clone)]
pub struct ScratchPad {
  pub dst: [Vec<FieldElement>; 3],
  pub twiddle_factor: Vec<FieldElement>,
  pub inv_twiddle_factor: Vec<FieldElement>,
  pub twiddle_factor_size: usize,
}

impl ScratchPad {
  pub fn from_order(order: usize) -> Self {
    let dst = [
      vec![FieldElement::default(); order],
      vec![FieldElement::default(); order],
      vec![FieldElement::default(); order],
    ];

    let mut twiddle_factor = Vec::with_capacity(order);
    let mut inv_twiddle_factor = Vec::with_capacity(order);

    let twiddle_factor_size = order;

    let rou = FieldElement::get_root_of_unity(my_log(order).expect("Log order not power of two"))
      .expect("Log order too high");

    let inv_rou = rou.inverse();

    twiddle_factor.push(FieldElement::real_one());
    inv_twiddle_factor.push(FieldElement::real_one());

    let twiddle_factor = std::iter::successors(Some(twiddle_factor[0]), |&prev| Some(rou * prev))
      .take(order)
      .collect::<Vec<_>>();

    let inv_twiddle_factor =
      std::iter::successors(Some(inv_twiddle_factor[0]), |&prev| Some(inv_rou * prev))
        .take(order)
        .collect::<Vec<_>>();

    ScratchPad {
      dst,
      twiddle_factor,
      inv_twiddle_factor,
      twiddle_factor_size,
    }
  }
}

pub fn fast_fourier_transform(
  coefficients: &[FieldElement],
  coefficient_len: usize,
  order: usize,
  root_of_unity: FieldElement,
  result: &mut [FieldElement],
  scratch_pad: &mut ScratchPad,
  twiddle_fac: Option<Vec<FieldElement>>,
) {
  assert_eq!(coefficient_len, coefficients.len());
  println!("pass");
  let twiddle_fac = twiddle_fac.unwrap_or_else(|| scratch_pad.twiddle_factor.clone());
  let mut rot_mul = Vec::with_capacity(MAX_ORDER_FFT);

  let mut log_order: Option<usize> = None;
  rot_mul.push(root_of_unity);

  for i in 1..MAX_ORDER_FFT {
    rot_mul.push(rot_mul[i - 1] * rot_mul[i - 1]);

    if (1usize << i) == order {
      log_order = Some(i);
    }
  }

  let mut log_coefficient: Option<usize> = None;

  if coefficient_len.is_power_of_two() {
    log_coefficient = Some(coefficient_len.trailing_zeros() as usize);
  }

  assert!(log_order.is_some());
  assert!(log_coefficient.is_some());

  let log_order = log_order.expect("Expected log_order to have a value");
  assert_eq!(rot_mul[log_order], FieldElement::real_one());

  let log_coefficient = log_coefficient.expect("Expected log_coefficient to have a value");
  assert!(log_coefficient <= log_order);

  // initialize leaves
  let blk_sz = order / coefficient_len;

  (0..blk_sz).for_each(|j| {
    let dst_base = j << log_coefficient;
    (0..coefficient_len).for_each(|i| {
      let dst_index = dst_base | i;
      scratch_pad.dst[log_coefficient & 1][dst_index] = coefficients[i];
    });
  });

  {
    // initialize leaves
    {
      let twiddle_size = scratch_pad.twiddle_factor_size;
      for dep in (0..log_coefficient).rev() {
        let blk_size = 1 << (log_order - dep);
        let half_blk_size = blk_size >> 1;
        let cur = dep & 1;
        let pre = cur ^ 1;
        //Todo: check this
        let pre_ptr = scratch_pad.dst[pre].clone();
        let cur_ptr = &mut scratch_pad.dst[cur];
        let gap = (twiddle_size / order) * (1 << (dep));
        assert_eq!(twiddle_size % order, 0);
        {
          for k in 0..blk_size / 2 {
            let double_k = (k) & (half_blk_size - 1);
            let x = twiddle_fac[k * gap];
            for j in 0..(1 << dep) {
              let l_value = pre_ptr[(double_k << (dep + 1)) | j];
              let r_value = x * pre_ptr[(double_k << (dep + 1) | (1 << dep)) | j];
              cur_ptr[(k << dep) | j] = l_value + r_value;
              cur_ptr[((k + blk_size / 2) << dep) | j] = l_value - r_value;
            }
          }
        }
      }
    }
  }

  result[..order].copy_from_slice(&scratch_pad.dst[0][..order]);
}

pub fn inverse_fast_fourier_transform(
  scratch_pad: &mut ScratchPad,
  evaluations: &[FieldElement],
  mut coefficient_len: usize,
  mut order: usize,
  root_of_unity: FieldElement,
  dst: &mut [FieldElement],
) {
  if coefficient_len > order {
    let error_message = format!(
      "Got insufficient number {} of evaluations for inverse fast fourier transform. Creating \
         polynomial of order {} instead.",
      coefficient_len, order
    );
    eprintln!("{}", error_message);
    coefficient_len = order;
  }

  let sub_eval: Vec<FieldElement> = if coefficient_len != order {
    (0..coefficient_len)
      .map(|i| evaluations[i * (order / coefficient_len)])
      .collect()
  } else {
    evaluations.to_vec()
  };

  let mut new_rou = FieldElement::real_one();
  for _ in 0..(order / coefficient_len) {
    new_rou = new_rou * root_of_unity;
  }
  order = coefficient_len;

  let mut inv_rou = FieldElement::real_one();
  let mut tmp = new_rou;

  let mut log_order: Option<usize> = None;

  if order.is_power_of_two() {
    log_order = Some(order.trailing_zeros() as usize);
  }

  let mut log_coefficient: Option<usize> = None;
  for i in 0..MAX_ORDER_FFT {
    if (1usize << i) == coefficient_len {
      log_coefficient = Some(i);
      break;
    }
  }

  assert!(log_order.is_some() && log_coefficient.is_some());
  let log_order = log_order.expect("log_order expected to have a value");

  for _ in 0..log_order {
    inv_rou = inv_rou * tmp;
    tmp = tmp * tmp;
  }
  assert_eq!(inv_rou * new_rou, FieldElement::real_one());
  println!("first");
  fast_fourier_transform(
    &sub_eval,
    order,
    coefficient_len,
    inv_rou,
    dst,
    scratch_pad,
    Some(scratch_pad.inv_twiddle_factor.clone()),
  );

  let inv_n = FieldElement::inverse(FieldElement::from_real(order as u64));
  assert_eq!(
    inv_n * FieldElement::from_real(order as u64),
    FieldElement::from_real(1)
  );

  (0..coefficient_len).for_each(|i| {
    dst[i] = dst[i] * inv_n;
  });
}
