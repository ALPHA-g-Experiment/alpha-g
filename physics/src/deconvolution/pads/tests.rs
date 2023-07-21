use super::*;

#[test]
fn trivial_single_pad_deconvolution() {
    let scale = 55.0;
    let signal = PAD_RESPONSE.iter().map(|x| x * scale).collect::<Vec<_>>();

    let deconvolved = pad_deconvolution(&signal);
    for (time, sample) in deconvolved.iter().enumerate() {
        if time == 0 {
            let diff = sample - scale;
            assert!(diff.abs() < 1e-6);
        } else {
            assert!(sample.abs() < 1e-6);
        }
    }
}
