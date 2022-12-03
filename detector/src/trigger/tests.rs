use super::*;

const TRG_V3_PACKET: [u8; 80] = [
    255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0,
    0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0,
    0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224,
];

#[test]
fn trg_v3_good() {
    let mut packet = TRG_V3_PACKET;
    for i in 0..=252 {
        packet[4] = i;
        packet[12] = i;
        packet[76] = i;

        packet[44] = i + 1;
        packet[40] = i + 2;
        packet[16] = i + 3;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }

    for i in 0..=u8::MAX {
        packet[24] = i;
        packet[25] = i;
        packet[26] = i;
        packet[27] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }

    for i in 0..=u8::MAX {
        packet[28] = i;
        packet[29] = i;
        packet[30] = i;
        packet[31] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }

    for i in 0..=u8::MAX {
        packet[32] = i;
        packet[33] = i;
        packet[34] = i;
        packet[35] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }

    for i in 0..=u8::MAX {
        packet[36] = i;
        packet[37] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }
    packet[39] = 0;
    assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    packet[39] = 128;
    assert!(TrgV3Packet::try_from(&packet[..]).is_ok());

    for i in 0..=u8::MAX {
        packet[52] = i;
        packet[53] = i;
        packet[54] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }

    for i in 0..=u8::MAX {
        packet[56] = i;
        packet[57] = i;
        packet[58] = i;
        packet[59] = i;
        packet[60] = i;
        packet[61] = i;
        packet[62] = i;
        packet[63] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }

    for i in 0..=u8::MAX {
        packet[64] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }

    for i in 0..=u8::MAX {
        packet[68] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }

    for i in 0..=u8::MAX {
        packet[72] = i;
        packet[73] = i;
        packet[74] = i;
        packet[75] = i;
        assert!(TrgV3Packet::try_from(&packet[..]).is_ok());
    }
}

#[test]
fn trg_v3_packet_slice_length_mismatch() {
    let mut bad_packet = TRG_V3_PACKET.to_vec();
    bad_packet.push(0);
    match TrgV3Packet::try_from(&bad_packet[..]) {
        Err(TryTrgPacketFromSliceError::SliceLengthMismatch { found, expected }) => {
            assert_eq!(found, 81);
            assert_eq!(expected, 80);
        }
        _ => unreachable!(),
    }

    let bad_packet = &TRG_V3_PACKET[..79];
    match TrgV3Packet::try_from(bad_packet) {
        Err(TryTrgPacketFromSliceError::SliceLengthMismatch { found, expected }) => {
            assert_eq!(found, 79);
            assert_eq!(expected, 80);
        }
        _ => unreachable!(),
    }
}

#[test]
fn trg_v3_packet_header_mask_mismatch() {
    let mut bad_packet = TRG_V3_PACKET;
    for i in 0..=16u32 {
        if i == 8 {
            continue;
        }
        bad_packet[7] = (i as u8) << 4;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::HeaderMaskMismatch { found }) => {
                assert_eq!(found, i << 28);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn trg_v3_packet_footer_mask_mismatch() {
    let mut bad_packet = TRG_V3_PACKET;
    for i in 0..=16u32 {
        if i == 14 {
            continue;
        }
        bad_packet[79] = (i as u8) << 4;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::FooterMaskMismatch { found }) => {
                assert_eq!(found, i << 28);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn trg_v3_packet_trig_out_mismatch() {
    let mut bad_packet = TRG_V3_PACKET;
    for i in 1..255u32 {
        bad_packet[4] = i as u8;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::TrigOutMismatch {
                header,
                footer,
                value,
            }) => {
                assert_eq!(header, i);
                assert_eq!(footer, 0);
                assert_eq!(value, 0);
            }
            _ => unreachable!(),
        }
    }

    let mut bad_packet = TRG_V3_PACKET;
    for i in 1..255u32 {
        bad_packet[76] = i as u8;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::TrigOutMismatch {
                header,
                footer,
                value,
            }) => {
                assert_eq!(header, 0);
                assert_eq!(footer, i);
                assert_eq!(value, 0);
            }
            _ => unreachable!(),
        }
    }

    let mut bad_packet = TRG_V3_PACKET;
    for i in 1..255u32 {
        bad_packet[12] = i as u8;
        bad_packet[16] = i as u8;
        bad_packet[40] = i as u8;
        bad_packet[44] = i as u8;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::TrigOutMismatch {
                header,
                footer,
                value,
            }) => {
                assert_eq!(header, 0);
                assert_eq!(footer, 0);
                assert_eq!(value, i);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn trg_v3_packet_bad_trig_in() {
    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[12] = 255;
    for i in 1..255u32 {
        bad_packet[16] = i as u8;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::BadTrigIn {
                found,
                min_expected,
            }) => {
                assert_eq!(found, i);
                assert_eq!(min_expected, 255);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn trg_v3_packet_bad_drift_counter() {
    let mut bad_packet = TRG_V3_PACKET;
    for i in 4..256u32 {
        bad_packet[40] = i as u8;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::BadDriftCounter {
                found,
                min_expected,
                max_expected,
            }) => {
                assert_eq!(found, i);
                assert_eq!(min_expected, 0);
                assert_eq!(max_expected, 3);
            }
            _ => unreachable!(),
        }
    }

    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[12] = 255;
    bad_packet[16] = 255;
    for i in 0..255u32 {
        bad_packet[40] = i as u8;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::BadDriftCounter {
                found,
                min_expected,
                max_expected,
            }) => {
                assert_eq!(found, i);
                assert_eq!(min_expected, 255);
                assert_eq!(max_expected, 255);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn trg_v3_packet_bad_scaledown_counter() {
    let mut bad_packet = TRG_V3_PACKET;
    for i in 3..256u32 {
        bad_packet[44] = i as u8;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::BadScaledownCounter {
                found,
                min_expected,
                max_expected,
            }) => {
                assert_eq!(found, i);
                assert_eq!(min_expected, 0);
                assert_eq!(max_expected, 2);
            }
            _ => unreachable!(),
        }
    }

    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[12] = 255;
    bad_packet[16] = 255;
    bad_packet[40] = 255;
    for i in 0..255u32 {
        bad_packet[44] = i as u8;
        match TrgV3Packet::try_from(&bad_packet[..]) {
            Err(TryTrgPacketFromSliceError::BadScaledownCounter {
                found,
                min_expected,
                max_expected,
            }) => {
                assert_eq!(found, i);
                assert_eq!(min_expected, 255);
                assert_eq!(max_expected, 255);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn trg_v3_packet_zero_mismatch() {
    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[3] = 128;
    match TrgV3Packet::try_from(&bad_packet[..]) {
        Err(TryTrgPacketFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, 2147483648);
        }
        _ => unreachable!(),
    }

    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[39] = 64;
    match TrgV3Packet::try_from(&bad_packet[..]) {
        Err(TryTrgPacketFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, 1073741824);
        }
        _ => unreachable!(),
    }

    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[51] = 128;
    match TrgV3Packet::try_from(&bad_packet[..]) {
        Err(TryTrgPacketFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, 2147483648);
        }
        _ => unreachable!(),
    }

    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[55] = 128;
    match TrgV3Packet::try_from(&bad_packet[..]) {
        Err(TryTrgPacketFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, 2147483648);
        }
        _ => unreachable!(),
    }

    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[67] = 128;
    match TrgV3Packet::try_from(&bad_packet[..]) {
        Err(TryTrgPacketFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, 2147483648);
        }
        _ => unreachable!(),
    }

    let mut bad_packet = TRG_V3_PACKET;
    bad_packet[71] = 128;
    match TrgV3Packet::try_from(&bad_packet[..]) {
        Err(TryTrgPacketFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, 2147483648);
        }
        _ => unreachable!(),
    }
}

#[test]
fn trg_v3_packet_udp_counter() {
    for i in 0..3 {
        let mut packet = TRG_V3_PACKET;
        packet[0] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.udp_counter(), (j as u32) << (8 * i));
        }
    }
}

#[test]
fn trg_v3_packet_timestamp() {
    for i in 8..12 {
        let mut packet = TRG_V3_PACKET;
        packet[8] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.timestamp(), (j as u32) << (8 * (i - 8)));
        }
    }
}

#[test]
fn trg_v3_packet_output_counter() {
    for i in 12..15 {
        let mut packet = TRG_V3_PACKET;
        for j in 0..u8::MAX {
            packet[16] = 0;
            packet[40] = 0;
            packet[44] = 0;

            packet[i] = j;
            packet[i - 8] = j;
            packet[i + 64] = j;

            packet[i + 32] = j;
            packet[44] += 1;

            packet[i + 28] = j;
            packet[40] += 1;

            packet[i + 4] = j;
            packet[16] += 1;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.output_counter(), (j as u32) << (8 * (i - 12)));
        }
    }
}

#[test]
fn trg_v3_packet_input_counter() {
    for i in 16..20 {
        let mut packet = TRG_V3_PACKET;
        packet[16] = 0;
        for j in 2..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.input_counter(), (j as u32) << (8 * (i - 16)));
        }
    }
}

#[test]
fn trg_v3_packet_pulser_counter() {
    for i in 20..24 {
        let mut packet = TRG_V3_PACKET;
        packet[20] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.pulser_counter(), (j as u32) << (8 * (i - 20)));
        }
    }
}

#[test]
fn trg_v3_packet_trigger_bitmap() {
    for i in 24..28 {
        let mut packet = TRG_V3_PACKET;
        packet[24] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.trigger_bitmap(), (j as u32) << (8 * (i - 24)));
        }
    }
}

#[test]
fn trg_v3_packet_nim_bitmap() {
    for i in 28..32 {
        let mut packet = TRG_V3_PACKET;
        packet[28] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.nim_bitmap(), (j as u32) << (8 * (i - 28)));
        }
    }
}

#[test]
fn trg_v3_packet_esata_bitmap() {
    for i in 32..36 {
        let mut packet = TRG_V3_PACKET;
        packet[32] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.esata_bitmap(), (j as u32) << (8 * (i - 32)));
        }
    }
}

#[test]
fn trg_v3_packet_satisfied_mlu() {
    let mut packet = TRG_V3_PACKET;
    packet[39] = 128;
    let trg_packet = TrgV3Packet::try_from(&packet[..]).unwrap();
    assert!(trg_packet.satisfied_mlu());

    packet[39] = 0;
    let trg_packet = TrgV3Packet::try_from(&packet[..]).unwrap();
    assert!(!trg_packet.satisfied_mlu());
}

#[test]
fn trg_v3_packet_aw16_prompt() {
    for i in 36..38 {
        let mut packet = TRG_V3_PACKET;
        packet[36] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.aw16_prompt(), (j as u16) << (8 * (i - 36)));
        }
    }
}

#[test]
fn trg_v3_packet_drift_veto_counter() {
    for i in 40..44 {
        let mut packet = TRG_V3_PACKET;
        packet[40] = 0;
        for j in 1..u8::MAX {
            packet[16] = 0;

            packet[i] = j;

            packet[i - 24] = j;
            packet[16] += 1;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.drift_veto_counter(), (j as u32) << (8 * (i - 40)));
        }
    }
}

#[test]
fn trg_v3_packet_scaledown_counter() {
    for i in 44..48 {
        let mut packet = TRG_V3_PACKET;
        packet[44] = 0;
        for j in 0..u8::MAX {
            packet[16] = 0;
            packet[40] = 0;

            packet[i] = j;

            packet[i - 28] = j;
            packet[16] += 1;

            packet[i - 4] = j;
            packet[40] += 1;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.scaledown_counter(), (j as u32) << (8 * (i - 44)));
        }
    }
}

#[test]
fn trg_v3_packet_aw16_multiplicity() {
    let mut packet = TRG_V3_PACKET;
    for i in 0..=u8::MAX {
        packet[54] = i;
        let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
        assert_eq!(packet.aw16_multiplicity(), i);
    }
}

#[test]
fn trg_v3_packet_aw16_bus() {
    for i in 52..54 {
        let mut packet = TRG_V3_PACKET;
        packet[52] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.aw16_bus(), (j as u16) << (8 * (i - 52)));
        }
    }
}

#[test]
fn trg_v3_packet_bsc64_bus() {
    for i in 56..64 {
        let mut packet = TRG_V3_PACKET;
        packet[56] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.bsc64_bus(), (j as u64) << (8 * (i - 56)));
        }
    }
}

#[test]
fn trg_v3_packet_bsc64_multiplicity() {
    let mut packet = TRG_V3_PACKET;
    for i in 0..=u8::MAX {
        packet[64] = i;
        let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
        assert_eq!(packet.bsc64_multiplicity(), i);
    }
}

#[test]
fn trg_v3_packet_coincidence_latch() {
    let mut packet = TRG_V3_PACKET;
    for i in 0..=u8::MAX {
        packet[68] = i;
        let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
        assert_eq!(packet.coincidence_latch(), i);
    }
}

#[test]
fn trg_v3_packet_firmware_revision() {
    for i in 72..76 {
        let mut packet = TRG_V3_PACKET;
        packet[72] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgV3Packet::try_from(&packet[..]).unwrap();
            assert_eq!(packet.firmware_revision(), (j as u32) << (8 * (i - 72)));
        }
    }
}

#[test]
fn trg_packet_udp_counter() {
    for i in 0..3 {
        let mut packet = TRG_V3_PACKET;
        packet[0] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.udp_counter(), (j as u32) << (8 * i));
        }
    }
}

#[test]
fn trg_packet_timestamp() {
    for i in 8..12 {
        let mut packet = TRG_V3_PACKET;
        packet[8] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.timestamp(), (j as u32) << (8 * (i - 8)));
        }
    }
}

#[test]
fn trg_packet_output_counter() {
    for i in 12..15 {
        let mut packet = TRG_V3_PACKET;
        for j in 0..u8::MAX {
            packet[16] = 0;
            packet[40] = 0;
            packet[44] = 0;

            packet[i] = j;
            packet[i - 8] = j;
            packet[i + 64] = j;

            packet[i + 32] = j;
            packet[44] += 1;

            packet[i + 28] = j;
            packet[40] += 1;

            packet[i + 4] = j;
            packet[16] += 1;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.output_counter(), (j as u32) << (8 * (i - 12)));
        }
    }
}

#[test]
fn trg_packet_input_counter() {
    for i in 16..20 {
        let mut packet = TRG_V3_PACKET;
        packet[16] = 0;
        for j in 2..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.input_counter(), (j as u32) << (8 * (i - 16)));
        }
    }
}

#[test]
fn trg_packet_pulser_counter() {
    for i in 20..24 {
        let mut packet = TRG_V3_PACKET;
        packet[20] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.pulser_counter(), (j as u32) << (8 * (i - 20)));
        }
    }
}

#[test]
fn trg_packet_trigger_bitmap() {
    for i in 24..28 {
        let mut packet = TRG_V3_PACKET;
        packet[24] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.trigger_bitmap(), (j as u32) << (8 * (i - 24)));
        }
    }
}

#[test]
fn trg_packet_nim_bitmap() {
    for i in 28..32 {
        let mut packet = TRG_V3_PACKET;
        packet[28] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.nim_bitmap(), (j as u32) << (8 * (i - 28)));
        }
    }
}

#[test]
fn trg_packet_esata_bitmap() {
    for i in 32..36 {
        let mut packet = TRG_V3_PACKET;
        packet[32] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.esata_bitmap(), (j as u32) << (8 * (i - 32)));
        }
    }
}

#[test]
fn trg_packet_satisfied_mlu() {
    let mut packet = TRG_V3_PACKET;
    packet[39] = 128;
    let trg_packet = TrgPacket::try_from(&packet[..]).unwrap();
    assert!(trg_packet.satisfied_mlu().unwrap());

    packet[39] = 0;
    let trg_packet = TrgPacket::try_from(&packet[..]).unwrap();
    assert!(!trg_packet.satisfied_mlu().unwrap());
}

#[test]
fn trg_packet_aw16_prompt() {
    for i in 36..38 {
        let mut packet = TRG_V3_PACKET;
        packet[36] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.aw16_prompt().unwrap(), (j as u16) << (8 * (i - 36)));
        }
    }
}

#[test]
fn trg_packet_drift_veto_counter() {
    for i in 40..44 {
        let mut packet = TRG_V3_PACKET;
        packet[40] = 0;
        for j in 1..u8::MAX {
            packet[16] = 0;

            packet[i] = j;

            packet[i - 24] = j;
            packet[16] += 1;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(
                packet.drift_veto_counter().unwrap(),
                (j as u32) << (8 * (i - 40))
            );
        }
    }
}

#[test]
fn trg_packet_scaledown_counter() {
    for i in 44..48 {
        let mut packet = TRG_V3_PACKET;
        packet[44] = 0;
        for j in 0..u8::MAX {
            packet[16] = 0;
            packet[40] = 0;

            packet[i] = j;

            packet[i - 28] = j;
            packet[16] += 1;

            packet[i - 4] = j;
            packet[40] += 1;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(
                packet.scaledown_counter().unwrap(),
                (j as u32) << (8 * (i - 44))
            );
        }
    }
}

#[test]
fn trg_packet_aw16_multiplicity() {
    let mut packet = TRG_V3_PACKET;
    for i in 0..=u8::MAX {
        packet[54] = i;
        let packet = TrgPacket::try_from(&packet[..]).unwrap();
        assert_eq!(packet.aw16_multiplicity().unwrap(), i);
    }
}

#[test]
fn trg_packet_aw16_bus() {
    for i in 52..54 {
        let mut packet = TRG_V3_PACKET;
        packet[52] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.aw16_bus().unwrap(), (j as u16) << (8 * (i - 52)));
        }
    }
}

#[test]
fn trg_packet_bsc64_bus() {
    for i in 56..64 {
        let mut packet = TRG_V3_PACKET;
        packet[56] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(packet.bsc64_bus().unwrap(), (j as u64) << (8 * (i - 56)));
        }
    }
}

#[test]
fn trg_packet_bsc64_multiplicity() {
    let mut packet = TRG_V3_PACKET;
    for i in 0..=u8::MAX {
        packet[64] = i;
        let packet = TrgPacket::try_from(&packet[..]).unwrap();
        assert_eq!(packet.bsc64_multiplicity().unwrap(), i);
    }
}

#[test]
fn trg_packet_coincidence_latch() {
    let mut packet = TRG_V3_PACKET;
    for i in 0..=u8::MAX {
        packet[68] = i;
        let packet = TrgPacket::try_from(&packet[..]).unwrap();
        assert_eq!(packet.coincidence_latch().unwrap(), i);
    }
}

#[test]
fn trg_packet_firmware_revision() {
    for i in 72..76 {
        let mut packet = TRG_V3_PACKET;
        packet[72] = 0;
        for j in 0..=u8::MAX {
            packet[i] = j;

            let packet = TrgPacket::try_from(&packet[..]).unwrap();
            assert_eq!(
                packet.firmware_revision().unwrap(),
                (j as u32) << (8 * (i - 72))
            );
        }
    }
}
