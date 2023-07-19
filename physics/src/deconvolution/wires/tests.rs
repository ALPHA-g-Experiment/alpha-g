use super::*;

#[test]
fn find_contiguous_wires() {
    let mut signals = [(); TPC_ANODE_WIRES].map(|_| None);

    signals[0] = Some(vec![0.0]);
    let ranges = contiguous_ranges(&signals);
    assert_eq!(ranges.len(), 1);
    assert!(ranges.contains(&(0, 1)));

    signals[15] = Some(vec![0.0]);
    signals[16] = Some(vec![0.0]);
    let ranges = contiguous_ranges(&signals);
    assert_eq!(ranges.len(), 2);
    assert!(ranges.contains(&(0, 1)));
    assert!(ranges.contains(&(15, 17)));

    signals[250] = Some(vec![0.0]);
    signals[251] = Some(vec![0.0]);
    signals[252] = Some(vec![0.0]);
    signals[253] = Some(vec![0.0]);
    signals[254] = Some(vec![0.0]);
    signals[255] = Some(vec![0.0]);
    let ranges = contiguous_ranges(&signals);
    assert_eq!(ranges.len(), 2);
    assert!(ranges.contains(&(250, 1)));
    assert!(ranges.contains(&(15, 17)));
}

#[test]
fn trivial_single_wire_deconvolution() {
    let mut signals = [(); TPC_ANODE_WIRES].map(|_| None);

    let scale = 80.0;
    let signal = vec![0.0; 10]
        .into_iter()
        .chain(
            WIRE_RESPONSE
                .iter()
                .map(|x| x * scale * NEIGHBOR_FACTORS[0]),
        )
        .collect::<Vec<_>>();
    signals[0] = Some(signal);

    let deconvolved = wire_range_deconvolution(&signals, (0, 1));
    for (channel, recovered) in deconvolved.iter().enumerate() {
        for (t, sample) in recovered.iter().enumerate() {
            if channel == 0 && t == 10 {
                let diff = sample - scale;
                assert!(diff.abs() < 1e-6);
            } else {
                assert!(sample.abs() < 1e-6);
            }
        }
    }
}

#[test]
fn trivial_multiple_wires_deconvolution() {
    let mut signals = [(); TPC_ANODE_WIRES].map(|_| None);

    let scale = 80.0;
    let offset = 10;
    for (i, factor) in NEIGHBOR_FACTORS.iter().enumerate() {
        let signal = vec![0.0; offset]
            .into_iter()
            .chain(WIRE_RESPONSE.iter().map(|x| x * scale * factor))
            .collect::<Vec<_>>();
        signals[i] = Some(signal.clone());
        if i > 0 {
            signals[TPC_ANODE_WIRES - i] = Some(signal);
        }
    }

    let deconvolved = wire_range_deconvolution(&signals, (252, 5));
    for (channel, recovered) in deconvolved.iter().enumerate() {
        for (t, sample) in recovered.iter().enumerate() {
            if channel == 4 && t == offset {
                let diff = sample - scale;
                assert!(diff.abs() < 1e-6);
            } else {
                assert!(sample.abs() < 1e-6);
            }
        }
    }
}
