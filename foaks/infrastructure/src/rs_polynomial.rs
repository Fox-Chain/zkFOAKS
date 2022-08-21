use std::borrow::Cow;

use crate::utility::my_log;
use linear_gkr::prime_field::FieldElement;

static mut STATIC_DST: [Vec<FieldElement>; 3] = [Vec::new(), Vec::new(), Vec::new()];
static mut STATIC_TWIDDLE_FACTOR: Vec<FieldElement> = Vec::new();
static mut STATIC_INV_TWIDDLE_FACTOR: Vec<FieldElement> = Vec::new();

const MAX_ORDER: usize = 28;

static mut STATIC_TWIDDLE_FACTOR_SIZE: usize = 0;

/// Safety:
/// This function should only be called if it is certain that
/// no other thread is accessing the used data concurrently, up
/// until the function has completed
///
/// Used data:
/// dst
/// twiddle_factor
/// inv_twiddle_factor
/// twiddle_factor_size
unsafe fn init_scratch_pad(order: usize) {
    STATIC_DST[0].reserve(order);
    STATIC_DST[1].reserve(order);
    STATIC_DST[2].reserve(order);

    STATIC_TWIDDLE_FACTOR.reserve(order);
    STATIC_TWIDDLE_FACTOR_SIZE = order;
    STATIC_INV_TWIDDLE_FACTOR.reserve(order);

    let rou = FieldElement::get_root_of_unity(my_log(order).expect("Log order not power of two"))
        .expect("Log order too high");
    let inv_rou = rou.inverse();

    STATIC_TWIDDLE_FACTOR.push(FieldElement::real_one());
    STATIC_INV_TWIDDLE_FACTOR.push(FieldElement::real_one());

    let mut prev_twiddle_factor = STATIC_TWIDDLE_FACTOR[0];
    let mut prev_inv_twiddle_factor = STATIC_INV_TWIDDLE_FACTOR[0];
    for _ in 1..order {
        prev_twiddle_factor = rou * prev_twiddle_factor;
        prev_inv_twiddle_factor = inv_rou * prev_inv_twiddle_factor;

        STATIC_TWIDDLE_FACTOR.push(prev_twiddle_factor);
        STATIC_INV_TWIDDLE_FACTOR.push(prev_inv_twiddle_factor);
    }
}

/// Safety: Same as init_scratch_pad
unsafe fn delete_scratch_pad() {
    std::mem::take(&mut STATIC_DST[0]);
    std::mem::take(&mut STATIC_DST[1]);
    std::mem::take(&mut STATIC_TWIDDLE_FACTOR);
}

/// Safety:
///
/// Accesses to dst and twiddle_factor_size by this function may
/// only occur if no other threads are accessing them concurrently
unsafe fn fast_fourier_transform(
    coefficients: &[FieldElement],
    coefficient_len: usize,
    order: usize,
    root_of_unity: FieldElement,
    result: &mut [FieldElement],
    twiddle_fac: &mut [FieldElement],
) {
    let mut rot_mul: [FieldElement; MAX_ORDER] = [FieldElement::default(); MAX_ORDER];

    let mut log_order: Option<usize> = None;
    rot_mul[0] = root_of_unity;

    for i in 0..MAX_ORDER {
        if i > 0 {
            rot_mul[i] = rot_mul[i - 1] * rot_mul[i - 1];
        }

        if (1usize << i) == order {
            log_order = Some(i);
        }
    }

    let mut log_coefficient: Option<usize> = None;
    for i in 0..MAX_ORDER {
        if (1usize << i) == coefficient_len {
            log_coefficient = Some(i);
        }
    }

    assert!(log_order.is_some());
    assert!(log_coefficient.is_some());
    let log_order = log_order.unwrap();
    let log_coefficient = log_coefficient.unwrap();

    assert!(rot_mul[log_order] == FieldElement::real_one());
    assert!(log_coefficient <= log_order);

    let blk_sz = order / coefficient_len;
    for j in 0..blk_sz {
        for (i, coefficient) in coefficients
            .iter()
            .copied()
            .enumerate()
            .take(coefficient_len)
        {
            // SAFETY:
            // TODO
            //
            // This should be fine as long as nobody else accesses this memory while
            // the current function does, but we should consider doing the outer loop
            // as a parallel for if it is possible - that is, each j loop should
            // undoubtedly be accessing disjoint memory.
            STATIC_DST[log_coefficient & 1][(j << log_coefficient) | i] = coefficient;
        }

        {
            let twiddle_size = STATIC_TWIDDLE_FACTOR_SIZE;
            for dep in (log_coefficient - 1)..=0 {
                let blk_size = 1 << (log_order - dep);
                let half_blk_size = blk_size >> 1;
                let cur = dep & 1;
                let pre = cur ^ 1;

                let cur_ptr = &mut STATIC_DST[cur];
                let pre_ptr = &mut STATIC_DST[pre];

                assert!(!std::ptr::eq(cur_ptr, pre_ptr));

                let gap = (twiddle_size / order) * (1 << dep);
                assert!(twiddle_size % order == 0);
                {
                    for k in 0..(blk_size / 2) {
                        let double_k = k & (half_blk_size - 1);
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

    result[..order].copy_from_slice(&STATIC_DST[0][..order]);
}

pub unsafe fn inverse_fast_fourier_transform(
    evaluations: &[FieldElement],
    mut coefficient_len: usize,
    mut order: usize,
    root_of_unity: FieldElement,
    dst: &mut [FieldElement],
) {
    if coefficient_len > order {
        eprintln!("Got insufficient number {} of evaluations for inverse fast fourier transform. Creating polynomial of order {} instead.", coefficient_len, order);
        coefficient_len = order;
    }

    let sub_eval: Cow<[FieldElement]> = {
        if coefficient_len != order {
            let mut sub_eval = Vec::with_capacity(coefficient_len);

            for i in 0..coefficient_len {
                sub_eval[i] = evaluations[i * (order / coefficient_len)];
            }

            Cow::Owned(sub_eval)
        } else {
            Cow::Borrowed(evaluations)
        }
    };

    let mut new_rou = FieldElement::real_one();
    for _ in 0..(order / coefficient_len) {
        new_rou = new_rou * root_of_unity;
    }
    order = coefficient_len;

    let mut inv_rou = FieldElement::real_one();
    let mut tmp = new_rou;

    let mut log_order: Option<usize> = None;
    for i in 0..MAX_ORDER {
        if (1usize << i) == order {
            log_order = Some(i);
            break;
        }
    }

    let mut log_coefficient: Option<usize> = None;
    for i in 0..MAX_ORDER {
        if (1usize << i) == coefficient_len {
            log_coefficient = Some(i);
            break;
        }
    }

    assert!(log_order.is_some());
    assert!(log_coefficient.is_some());

    let log_order = log_order.unwrap();

    for _ in 0..log_order {
        inv_rou = inv_rou * tmp;
        tmp = tmp * tmp;
    }
    assert!(inv_rou * inv_rou == FieldElement::real_one());

    fast_fourier_transform(
        &sub_eval[..],
        order,
        coefficient_len,
        inv_rou,
        dst,
        &mut STATIC_INV_TWIDDLE_FACTOR[..],
    );

    let inv_n = FieldElement::from_real(order.try_into().unwrap()).inverse();
    assert!(inv_n * FieldElement::from_real(order.try_into().unwrap()) == FieldElement::real_one());

    for item in dst.iter_mut().take(coefficient_len) {
        *item = *item * inv_n;
    }
}
