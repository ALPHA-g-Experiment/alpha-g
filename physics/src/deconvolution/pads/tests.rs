use super::*;

#[test]
fn trivial_single_pad_deconvolution() {
    std::thread::Builder::new()
        .stack_size(32 * 575 * 24 * 10)
        .spawn(|| {
            let mut pad_signals = [(); TPC_PAD_COLUMNS].map(|_| [(); TPC_PAD_ROWS].map(|_| None));

            let scale = 55.0;
            let signal = PAD_RESPONSE.iter().map(|x| x * scale).collect::<Vec<_>>();
            pad_signals[0][0] = Some(signal);

            let deconvolved = pad_deconvolution(&pad_signals, 0, 0);
            for (time, sample) in deconvolved.iter().enumerate() {
                if time == 0 {
                    let diff = sample - scale;
                    assert!(diff.abs() < 1e-6);
                } else {
                    assert!(sample.abs() < 1e-6);
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
