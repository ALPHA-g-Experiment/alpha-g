use super::*;

#[test]
fn timestamp_bits() {
    assert_eq!(TIMESTAMP_BITS, 24);
}

#[test]
fn timestamp_clock_frequency() {
    assert_eq!(TIMESTAMP_CLOCK_FREQ, 10000000.0);
}

#[test]
fn try_chronobox_channel_id() {
    for num in 0..=58 {
        assert_eq!(ChannelId(num), ChannelId::try_from(num).unwrap());
    }
    for num in 59..=255 {
        assert!(ChannelId::try_from(num).is_err());
    }
}

#[test]
fn from_channel_id_u8() {
    for i in 0..=58 {
        let channel = ChannelId::try_from(i).unwrap();
        assert_eq!(u8::from(channel), i);
    }
}

fn timestamp_counter(channel: u8, timestamp: u32, edge: bool) -> u32 {
    0x80000000 | (u32::from(channel) << 24) | (timestamp & 0x00FFFFFE) | u32::from(edge)
}

fn wrap_around_marker(top_bit: bool, counter: u32) -> u32 {
    0xFF000000 | (u32::from(top_bit) << 23) | (counter & 0x007FFFFF)
}

const SCALERS_HEADER: u32 = 0xFE00003C;

#[test]
fn fifo_completely_invalid() {
    let bad_word = 0u32;
    let bytes = [
        &bad_word.to_le_bytes()[..],
        &SCALERS_HEADER.to_le_bytes()[..],
        &[0; 60 * 4],
    ]
    .concat();

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);

    assert!(fifo.is_empty());
    assert_eq!(input, &bytes[..]);
}

#[test]
fn fifo_initially_invalid_timestamp_counter() {
    let tsc = timestamp_counter(0, 0, false).to_le_bytes();

    let mut input = &tsc[..2];
    let fifo = chronobox_fifo(&mut input);
    assert!(fifo.is_empty());
    assert_eq!(input, &tsc[..2]);

    input = &tsc[..];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), 1);
    let FifoEntry::TimestampCounter(tsc) = fifo[0] else {
        unreachable!();
    };
    assert_eq!(tsc.channel, ChannelId::try_from(0).unwrap());
    assert_eq!(tsc.timestamp(), 0);
    assert!(matches!(tsc.edge, EdgeType::Leading));
    assert!(input.is_empty());
}

#[test]
fn fifo_finally_invalid_timestamp_counter() {
    let tsc_1 = timestamp_counter(0, 0, false);
    let tsc_2 = timestamp_counter(1, 1, true);

    let bytes = [&tsc_1.to_le_bytes()[..], &tsc_2.to_le_bytes()[..]].concat();
    let mut input = &bytes[..6];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), 1);
    let FifoEntry::TimestampCounter(tsc) = fifo[0] else {
        unreachable!();
    };
    assert_eq!(tsc.channel, ChannelId::try_from(0).unwrap());
    assert_eq!(tsc.timestamp(), 0);
    assert!(matches!(tsc.edge, EdgeType::Leading));
    assert_eq!(input, &bytes[4..6]);
}

#[test]
fn fifo_initially_invalid_wrap_around_counter() {
    let wap = wrap_around_marker(true, 0).to_le_bytes();

    let mut input = &wap[..2];
    let fifo = chronobox_fifo(&mut input);

    assert!(fifo.is_empty());
    assert_eq!(input, &wap[..2]);

    input = &wap[..];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), 1);
    let FifoEntry::WrapAroundMarker(wam) = fifo[0] else {
        unreachable!();
    };
    assert!(wam.timestamp_top_bit);
    assert_eq!(wam.wrap_around_counter(), 0);
    assert!(input.is_empty());
}

#[test]
fn fifo_finally_invalid_wrap_around_counter() {
    let wam_1 = wrap_around_marker(true, 0);
    let wam_2 = wrap_around_marker(false, 1);

    let bytes = [&wam_1.to_le_bytes()[..], &wam_2.to_le_bytes()[..]].concat();
    let mut input = &bytes[..6];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), 1);
    let FifoEntry::WrapAroundMarker(wam) = fifo[0] else {
        unreachable!();
    };
    assert!(wam.timestamp_top_bit);
    assert_eq!(wam.wrap_around_counter(), 0);
    assert_eq!(input, &bytes[4..6]);
}

#[test]
fn fifo_invalid_channel() {
    let good_tsc = timestamp_counter(58, 121, true);
    for channel in 59..=126 {
        let bad_tsc = timestamp_counter(channel, 0, false);
        let bytes = [
            &good_tsc.to_le_bytes()[..],
            &bad_tsc.to_le_bytes()[..],
            &SCALERS_HEADER.to_le_bytes()[..],
            &[0; 60 * 4],
        ]
        .concat();

        let mut input = &bytes[..];
        let fifo = chronobox_fifo(&mut input);

        assert_eq!(fifo.len(), 1);
        let FifoEntry::TimestampCounter(tsc) = fifo[0] else {
            unreachable!();
        };
        assert_eq!(tsc.channel, ChannelId::try_from(58).unwrap());
        assert_eq!(tsc.timestamp(), 120);
        assert!(matches!(tsc.edge, EdgeType::Trailing));
        assert_eq!(input, &bytes[4..]);
    }
}

#[test]
fn fifo_empty_slice() {
    let mut input = &[][..];
    let fifo = chronobox_fifo(&mut input);

    assert!(fifo.is_empty());
    assert!(input.is_empty());
}

#[test]
fn fifo_short_scalers() {
    let bytes = [&SCALERS_HEADER.to_le_bytes()[..], &[0; 60 * 4]].concat();

    let mut input = &bytes[..4];
    let fifo = chronobox_fifo(&mut input);
    assert!(fifo.is_empty());
    assert_eq!(input, &bytes[..4]);

    input = &bytes[..bytes.len() - 1];
    let fifo = chronobox_fifo(&mut input);
    assert!(fifo.is_empty());
    assert_eq!(input, &bytes[..bytes.len() - 1]);

    input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert!(fifo.is_empty());
    assert!(input.is_empty());
}

#[test]
fn fifo_long_scalers() {
    let bytes = [&SCALERS_HEADER.to_le_bytes()[..], &[0; 61 * 4]].concat();

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert!(fifo.is_empty());
    assert_eq!(input, &bytes[bytes.len() - 4..]);
}

#[test]
fn fifo_before_scalers() {
    let mut bytes = Vec::new();
    for i in 0..10 {
        for channel in 0..=58 {
            let tsc_le = timestamp_counter(channel, i, false);
            bytes.extend_from_slice(&tsc_le.to_le_bytes()[..]);

            let tsc_te = timestamp_counter(channel, i + 1, true);
            bytes.extend_from_slice(&tsc_te.to_le_bytes()[..]);
        }

        let wam = wrap_around_marker(i % 2 == 0, i);
        bytes.extend_from_slice(&wam.to_le_bytes()[..]);
    }
    bytes.extend_from_slice(&SCALERS_HEADER.to_le_bytes()[..]);
    bytes.extend_from_slice(&[0x01; 60 * 4]);

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), (59 * 2 + 1) * 10);
    assert!(input.is_empty());
}

#[test]
fn fifo_after_scalers() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&SCALERS_HEADER.to_le_bytes()[..]);
    bytes.extend_from_slice(&[0x01; 60 * 4]);
    for i in 0..10 {
        for channel in 0..=58 {
            let tsc_le = timestamp_counter(channel, i, false);
            bytes.extend_from_slice(&tsc_le.to_le_bytes()[..]);

            let tsc_te = timestamp_counter(channel, i + 1, true);
            bytes.extend_from_slice(&tsc_te.to_le_bytes()[..]);
        }

        let wam = wrap_around_marker(i % 2 == 0, i);
        bytes.extend_from_slice(&wam.to_le_bytes()[..]);
    }

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), (59 * 2 + 1) * 10);
    assert!(input.is_empty());
}

#[test]
fn fifo_between_scalers() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&SCALERS_HEADER.to_le_bytes()[..]);
    bytes.extend_from_slice(&[0x11; 60 * 4]);
    for i in 0..10 {
        for channel in 0..=58 {
            let tsc_le = timestamp_counter(channel, i, false);
            bytes.extend_from_slice(&tsc_le.to_le_bytes()[..]);

            let tsc_te = timestamp_counter(channel, i + 1, true);
            bytes.extend_from_slice(&tsc_te.to_le_bytes()[..]);
        }

        let wam = wrap_around_marker(i % 2 == 0, i);
        bytes.extend_from_slice(&wam.to_le_bytes()[..]);
    }
    bytes.extend_from_slice(&SCALERS_HEADER.to_le_bytes()[..]);
    bytes.extend_from_slice(&[0xFF; 60 * 4]);

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), (59 * 2 + 1) * 10);
    assert!(input.is_empty());
}

#[test]
fn fifo_good_complete_long_packet() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&SCALERS_HEADER.to_le_bytes()[..]);
    bytes.extend_from_slice(&[0x01; 60 * 4]);
    for _ in 0..100 {
        for i in 0..10 {
            for channel in 0..=58 {
                let tsc_le = timestamp_counter(channel, i, false);
                bytes.extend_from_slice(&tsc_le.to_le_bytes()[..]);

                let tsc_te = timestamp_counter(channel, i + 1, true);
                bytes.extend_from_slice(&tsc_te.to_le_bytes()[..]);
            }

            let wam = wrap_around_marker(i % 2 == 0, i);
            bytes.extend_from_slice(&wam.to_le_bytes()[..]);
        }
        bytes.extend_from_slice(&SCALERS_HEADER.to_le_bytes()[..]);
        bytes.extend_from_slice(&[0x01; 60 * 4]);
    }

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), (59 * 2 + 1) * 10 * 100);
    assert!(input.is_empty());
}

#[test]
fn fifo_only_timestamp_counters() {
    let mut bytes = Vec::new();
    for i in 0..10 {
        for channel in 0..=58 {
            let tsc_le = timestamp_counter(channel, i, false);
            bytes.extend_from_slice(&tsc_le.to_le_bytes()[..]);

            let tsc_te = timestamp_counter(channel, i + 1, true);
            bytes.extend_from_slice(&tsc_te.to_le_bytes()[..]);
        }
    }

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), 59 * 2 * 10);
    assert!(input.is_empty());
}

#[test]
fn fifo_only_wrap_around_markers() {
    let mut bytes = Vec::new();
    for i in 0..10 {
        let wam = wrap_around_marker(i % 2 == 0, i);
        bytes.extend_from_slice(&wam.to_le_bytes()[..]);
    }

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert_eq!(fifo.len(), 10);
    assert!(input.is_empty());
}

#[test]
fn fifo_only_scalers_blocks() {
    let mut bytes = Vec::new();
    for _ in 0..10 {
        bytes.extend_from_slice(&SCALERS_HEADER.to_le_bytes()[..]);
        bytes.extend_from_slice(&[0x01; 60 * 4]);
    }

    let mut input = &bytes[..];
    let fifo = chronobox_fifo(&mut input);
    assert!(fifo.is_empty());
    assert!(input.is_empty());
}

#[test]
fn chronobox_board_id() {
    for name in ["01", "02", "03", "04"] {
        let board_id = BoardId::try_from(name).unwrap();
        assert_eq!(board_id.name(), name);
    }

    assert!(BoardId::try_from("00").is_err());
    assert!(BoardId::try_from("05").is_err());
}
