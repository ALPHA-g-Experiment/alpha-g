use super::*;

#[test]
fn try_adc_16_channel_id_from_u8() {
    for num in 0u8..=15u8 {
        assert_eq!(Adc16ChannelId::try_from(num).unwrap(), Adc16ChannelId(num));
    }
    for num in 16u8..=255u8 {
        assert!(Adc16ChannelId::try_from(num).is_err());
    }
}

#[test]
fn adc_16_channel_id_sampling_rate() {
    for num in 0u8..=15u8 {
        assert_eq!(Adc16ChannelId(num).sampling_rate(), 100e6);
    }
}

#[test]
fn try_adc_32_channel_id_from_u8() {
    for num in 0u8..=31u8 {
        assert_eq!(Adc32ChannelId::try_from(num).unwrap(), Adc32ChannelId(num));
    }
    for num in 32u8..=255u8 {
        assert!(Adc32ChannelId::try_from(num).is_err());
    }
}

#[test]
fn adc_32_channel_id_sampling_rate() {
    for num in 0u8..=31u8 {
        assert_eq!(Adc32ChannelId(num).sampling_rate(), 62.5e6);
    }
}

#[test]
fn channel_id_sampling_rate() {
    for num in 0..=15 {
        assert_eq!(
            ChannelId::A16(num.try_into().unwrap()).sampling_rate(),
            100e6
        );
    }
    for num in 0..=31 {
        assert_eq!(
            ChannelId::A32(num.try_into().unwrap()).sampling_rate(),
            62.5e6
        );
    }
}

#[test]
fn try_module_id_from_u8() {
    for num in 0u8..=7u8 {
        assert_eq!(ModuleId::try_from(num).unwrap(), ModuleId(num));
    }
    for num in 8u8..=255u8 {
        assert!(ModuleId::try_from(num).is_err());
    }
}

#[test]
fn alpha_16_boards() {
    for i in 0..ALPHA16BOARDS.len() {
        let next = i + 1;
        for j in next..ALPHA16BOARDS.len() {
            assert_ne!(ALPHA16BOARDS[i].0, ALPHA16BOARDS[j].0);
            assert_ne!(ALPHA16BOARDS[i].1, ALPHA16BOARDS[j].1);
        }
    }
}

#[test]
fn try_board_id_from_mac_address() {
    let board_id = BoardId::try_from([216, 128, 57, 104, 55, 76]).unwrap();
    assert_eq!(board_id.name(), "09");
    assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 55, 76]);

    let board_id = BoardId::try_from([216, 128, 57, 104, 170, 37]).unwrap();
    assert_eq!(board_id.name(), "10");
    assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 170, 37]);

    let board_id = BoardId::try_from([216, 128, 57, 104, 172, 127]).unwrap();
    assert_eq!(board_id.name(), "11");
    assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 172, 127]);

    let board_id = BoardId::try_from([216, 128, 57, 104, 79, 167]).unwrap();
    assert_eq!(board_id.name(), "12");
    assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 79, 167]);

    let board_id = BoardId::try_from([216, 128, 57, 104, 202, 166]).unwrap();
    assert_eq!(board_id.name(), "13");
    assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 202, 166]);

    let board_id = BoardId::try_from([216, 128, 57, 104, 142, 130]).unwrap();
    assert_eq!(board_id.name(), "14");
    assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 142, 130]);

    let board_id = BoardId::try_from([216, 128, 57, 104, 111, 162]).unwrap();
    assert_eq!(board_id.name(), "16");
    assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 111, 162]);

    let board_id = BoardId::try_from([216, 128, 57, 104, 142, 82]).unwrap();
    assert_eq!(board_id.name(), "18");
    assert_eq!(board_id.mac_address(), [216, 128, 57, 104, 142, 82]);
}

#[test]
fn board_id() {
    for pair in ALPHA16BOARDS {
        let board_id = BoardId::try_from(pair.1).unwrap();
        assert_eq!(board_id.name(), pair.0);
        assert_eq!(board_id.mac_address(), pair.1);
    }

    let board_id = BoardId::try_from([0, 0, 0, 0, 0, 0]);
    assert!(board_id.is_err());
}

const SHORT_ADC_V3_PACKET: [u8; 16] = [1, 3, 0, 1, 2, 3, 2, 187, 0, 0, 0, 4, 224, 0, 0, 0];
const LONG_ADC_V3_PACKET: [u8; 164] = [
    1, 3, 0, 1, 2, 3, 2, 187, 0, 0, 0, 4, 0, 0, 216, 128, 57, 104, 142, 82, 0, 0, 0, 0, 0, 0, 0, 5,
    0, 0, 0, 6, 255, 224, 255, 225, 255, 226, 255, 227, 255, 228, 255, 229, 255, 230, 255, 231,
    255, 232, 255, 233, 255, 234, 255, 235, 255, 236, 255, 237, 255, 238, 255, 239, 255, 240, 255,
    241, 255, 242, 255, 243, 255, 244, 255, 245, 255, 246, 255, 247, 255, 248, 255, 249, 255, 250,
    255, 251, 255, 252, 255, 253, 255, 254, 255, 255, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0,
    8, 0, 9, 0, 10, 0, 11, 0, 12, 0, 13, 0, 14, 0, 15, 0, 16, 0, 17, 0, 18, 0, 19, 0, 20, 0, 21, 0,
    22, 0, 23, 0, 24, 0, 25, 0, 26, 0, 27, 0, 28, 0, 29, 0, 30, 0, 31, 0, 32, 240, 33, 0, 0,
];

#[test]
fn adc_v3_good() {
    let mut good_packet = SHORT_ADC_V3_PACKET;
    for i in 0..=7 {
        good_packet[4] = i;
        assert!(AdcV3Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 0..=15 {
        good_packet[5] = i;
        assert!(AdcV3Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 128..=159 {
        good_packet[5] = i;
        assert!(AdcV3Packet::try_from(&good_packet[..]).is_ok());
    }

    let mut good_packet = LONG_ADC_V3_PACKET;
    for i in 0..=7 {
        good_packet[4] = i;
        assert!(AdcV3Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 0..=15 {
        good_packet[5] = i;
        assert!(AdcV3Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 128..=159 {
        good_packet[5] = i;
        assert!(AdcV3Packet::try_from(&good_packet[..]).is_ok());
    }
    for pair in ALPHA16BOARDS {
        good_packet[14..20].copy_from_slice(&pair.1[..]);
        assert!(AdcV3Packet::try_from(&good_packet[..]).is_ok());
    }
}

#[test]
fn adc_v3_packet_incomplete_slice() {
    let bad_packet = &SHORT_ADC_V3_PACKET[..SHORT_ADC_V3_PACKET.len() - 1];
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::IncompleteSlice)
    ));

    let bad_packet = &LONG_ADC_V3_PACKET[..30];
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::IncompleteSlice)
    ));
}

#[test]
fn adc_v3_packet_unknown_type() {
    let mut bad_packet = SHORT_ADC_V3_PACKET;
    bad_packet[0] = 10;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownType)
    ));

    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[0] = 10;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownType)
    ));
}

#[test]
fn adc_v3_packet_unknown_version() {
    let mut bad_packet = SHORT_ADC_V3_PACKET;
    bad_packet[1] = 9;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownVersion)
    ));

    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[1] = 9;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownVersion)
    ));
}

#[test]
fn adc_v3_packet_unknown_module_id() {
    let mut bad_packet = SHORT_ADC_V3_PACKET;
    bad_packet[4] = 9;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownModuleId)
    ));

    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[4] = 9;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownModuleId)
    ));
}

#[test]
fn adc_v3_packet_unknown_channel_id() {
    let mut bad_packet = SHORT_ADC_V3_PACKET;
    bad_packet[5] = 200;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownChannelId)
    ));

    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[5] = 50;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownChannelId)
    ));
}

#[test]
fn adc_v3_packet_zero_mismatch() {
    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[12] = 200;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::ZeroMismatch)
    ));

    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[13] = 50;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::ZeroMismatch)
    ));
}

#[test]
fn adc_v3_packet_unknown_mac() {
    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[14..20].copy_from_slice(&[0, 0, 0, 0, 0, 0]);
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::UnknownMac)
    ));
}

#[test]
fn adc_v3_packet_baseline_mismatch() {
    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[159] = 96;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::BaselineMismatch)
    ));
}

#[test]
fn adc_v3_packet_bad_keep_last() {
    let mut bad_packet = SHORT_ADC_V3_PACKET;
    bad_packet[13] = 1;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::BadKeepLast)
    ));

    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[161] = 34;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::BadKeepLast)
    ));
}

#[test]
fn adc_v3_packet_keep_bit_mismatch() {
    let mut bad_packet = SHORT_ADC_V3_PACKET;
    bad_packet[12] = 240;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::KeepBitMismatch)
    ));

    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[160] = 224;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::KeepBitMismatch)
    ));
}

#[test]
fn adc_v3_packet_number_of_samples_mismatch() {
    let mut bad_packet = SHORT_ADC_V3_PACKET;
    bad_packet[12] = 192;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::NumberOfSamplesMismatch)
    ));

    let mut bad_packet = LONG_ADC_V3_PACKET;
    bad_packet[160] = 208;
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::NumberOfSamplesMismatch)
    ));

    let packet = AdcV3Packet::try_from(&LONG_ADC_V3_PACKET[..]).unwrap();
    let requested_samples = packet.requested_samples();
    let waveform_samples = packet.waveform.len();

    let mut bad_packet: Vec<u8> = LONG_ADC_V3_PACKET.to_vec();
    for _ in 0..=requested_samples - waveform_samples {
        bad_packet.insert(32, 0);
        bad_packet.insert(32, 0);
    }

    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::NumberOfSamplesMismatch)
    ));
}

#[test]
fn adc_v3_packet_incomplete_waveform() {
    let mut bad_packet: Vec<u8> = LONG_ADC_V3_PACKET.to_vec();
    bad_packet.remove(32);
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::IncompleteWaveform)
    ));

    bad_packet.remove(32);
    assert!(matches!(
        AdcV3Packet::try_from(&bad_packet[..]),
        Err(TryAdcPacketFromSliceError::IncompleteWaveform)
    ));
}
