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
