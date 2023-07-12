// Identify all the contiguous 'Some' in a slice of `Option<T>`.
// Return an ordered vector with the (half-open) intervals of the first
// (inclusive) and the last (exclusive) indices in each contiguous block i.e.
// [first, last).
pub(crate) fn contiguous_ranges<T>(slice: &[Option<T>]) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut end = 0;
    while start < slice.len() {
        while end < slice.len() && slice[end].is_some() {
            end += 1;
        }
        if start < end {
            result.push((start, end));
        }
        start = end + 1;
        end = start;
    }

    result
}

// Non-negative Landweber iterations.
//
// Given a matrix equation rx = y, minimize |rx - y|^2 with non-negative
// constraints on x.
// Given our use case, the following characteristics of r can be assumed:
// 1. Square matrix
// 2. Lower triangular
// 3. Toeplitz
pub(crate) fn nn_landweber(
    r: faer_core::MatRef<f64>,
    y: faer_core::MatRef<f64>,
    omega: f64,
    iterations: usize,
) -> faer_core::Mat<f64> {
    let mut x = faer_core::Mat::zeros(y.nrows(), y.ncols());
    // Each iteration is:
    // x_{k+1} = x_k + omega * r.T * (y - r * x_k)
    for _ in 0..iterations {
        let mut y = y.to_owned();
        faer_core::mul::triangular::matmul_with_conj(
            y.as_mut(),
            faer_core::mul::triangular::BlockStructure::Rectangular,
            r,
            faer_core::mul::triangular::BlockStructure::TriangularLower,
            faer_core::Conj::No,
            x.as_ref(),
            faer_core::mul::triangular::BlockStructure::Rectangular,
            faer_core::Conj::No,
            Some(1.0),
            -1.0,
            faer_core::Parallelism::None,
        );
        faer_core::mul::triangular::matmul_with_conj(
            x.as_mut(),
            faer_core::mul::triangular::BlockStructure::Rectangular,
            r.transpose(),
            faer_core::mul::triangular::BlockStructure::TriangularUpper,
            faer_core::Conj::No,
            y.as_ref(),
            faer_core::mul::triangular::BlockStructure::Rectangular,
            faer_core::Conj::No,
            Some(1.0),
            omega,
            faer_core::Parallelism::None,
        );
        // Non-negative constraint
        x.as_mut().cwise().for_each(|mut x| {
            if x.read() < 0.0 {
                x.write(0.0)
            }
        });
    }

    x
}
