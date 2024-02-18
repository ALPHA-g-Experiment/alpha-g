use super::*;

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
fn chronopacket_scaler() {
    let packet = ChronoPacket {
        fifo: Vec::new(),
        scalers: (0..u32::try_from(NUM_INPUT_CHANNELS).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap(),
        sys_clock: 0,
    };

    for num in 0..=58 {
        let channel_id = ChannelId::try_from(num).unwrap();
        assert_eq!(packet.scaler(channel_id), u32::from(num));
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
fn chronopacket_invalid_top_bit() {
    let bad_tsc = timestamp_counter(0, 0, false) & 0x7FFFFFFF;
    let bytes = [
        &bad_tsc.to_le_bytes()[..],
        &SCALERS_HEADER.to_le_bytes()[..],
        &[0; 60 * 4],
    ]
    .concat();
    assert!(ChronoPacket::try_from(&bytes[..]).is_err());
}

#[test]
fn chronopacket_invalid_channel() {
    for channel in 59..=126 {
        let bad_tsc = timestamp_counter(channel, 0, false);
        let bytes = [
            &bad_tsc.to_le_bytes()[..],
            &SCALERS_HEADER.to_le_bytes()[..],
            &[0; 60 * 4],
        ]
        .concat();
        assert!(ChronoPacket::try_from(&bytes[..]).is_err());
    }
}

#[test]
fn chronopacket_unknown_word() {
    let tsc = timestamp_counter(0, 0, false);
    let wam = wrap_around_marker(false, 0);
    let unknown_word = 0xFC000000u32;
    let bytes = [
        &tsc.to_le_bytes()[..],
        &wam.to_le_bytes()[..],
        &unknown_word.to_le_bytes()[..],
        &SCALERS_HEADER.to_le_bytes()[..],
        &[0; 60 * 4],
    ]
    .concat();

    assert!(ChronoPacket::try_from(&bytes[..]).is_err());
}

#[test]
fn chronopacket_missing_scalers() {
    let bytes = Vec::new();
    assert!(ChronoPacket::try_from(&bytes[..]).is_err());
}

#[test]
fn chronopacket_short_scalers() {
    let bytes = [&SCALERS_HEADER.to_le_bytes()[..], &[0; 59 * 4]].concat();
    assert!(ChronoPacket::try_from(&bytes[..]).is_err());
}

#[test]
fn chronopacket_long_scalers() {
    let bytes = [&SCALERS_HEADER.to_le_bytes()[..], &[0; 61 * 4]].concat();
    assert!(ChronoPacket::try_from(&bytes[..]).is_err());
}

#[test]
fn chronopacket_good_single_timestamp_counter() {
    let tsc = timestamp_counter(11, 0, false);
    let bytes = [
        &tsc.to_le_bytes()[..],
        &SCALERS_HEADER.to_le_bytes()[..],
        &[0; 60 * 4],
    ]
    .concat();

    let packet = ChronoPacket::try_from(&bytes[..]).unwrap();
    let ChronoPacket {
        fifo,
        scalers,
        sys_clock,
    } = packet;

    assert!(fifo.len() == 1);
    let FifoEntry::TimestampCounter(tsc) = fifo[0] else {
        unreachable!();
    };
    assert_eq!(tsc.channel, ChannelId::try_from(11).unwrap());
    assert_eq!(tsc.timestamp(), 0);
    assert!(matches!(tsc.edge, EdgeType::Leading));
    assert_eq!(scalers, [0; 59]);
    assert_eq!(sys_clock, 0);
}

#[test]
fn chronopacket_good_single_wrap_around_marker() {
    let wam = wrap_around_marker(true, 0);
    let bytes = [
        &wam.to_le_bytes()[..],
        &SCALERS_HEADER.to_le_bytes()[..],
        &[0xFF; 60 * 4],
    ]
    .concat();

    let packet = ChronoPacket::try_from(&bytes[..]).unwrap();
    let ChronoPacket {
        fifo,
        scalers,
        sys_clock,
    } = packet;
    assert!(fifo.len() == 1);
    let FifoEntry::WrapAroundMarker(wam) = fifo[0] else {
        unreachable!();
    };
    assert!(wam.timestamp_top_bit);
    assert_eq!(wam.wrap_around_counter(), 0);
    assert_eq!(scalers, [0xFFFFFFFF; 59]);
    assert_eq!(sys_clock, 0xFFFFFFFF);
}

#[test]
fn chronopacket_good_only_scalers() {
    let bytes = [&SCALERS_HEADER.to_le_bytes()[..], &[0xFF; 60 * 4]].concat();
    let packet = ChronoPacket::try_from(&bytes[..]).unwrap();
    let ChronoPacket {
        fifo,
        scalers,
        sys_clock,
    } = packet;
    assert!(fifo.is_empty());
    assert_eq!(scalers, [0xFFFFFFFF; 59]);
    assert_eq!(sys_clock, 0xFFFFFFFF);
}

#[test]
fn chronopacket_good_full_packet() {
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

    let packet = ChronoPacket::try_from(&bytes[..]).unwrap();
    let ChronoPacket {
        fifo,
        scalers,
        sys_clock,
    } = packet;
    assert_eq!(fifo.len(), (59 * 2 + 1) * 10);
    assert_eq!(scalers, [0x01010101; 59]);
    assert_eq!(sys_clock, 0x01010101);
}
