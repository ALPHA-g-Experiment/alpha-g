use super::*;

#[test]
fn adc16_suppression_threshold_json_ptr() {
    assert_eq!(
        ADC16_SUPPRESSION_THRESHOLD_JSON_PTR,
        "/Equipment/CTRL/Settings/ADC/adc16_sthreshold"
    );
}

#[test]
fn adc32_suppression_enable_json_ptr() {
    assert_eq!(
        ADC32_SUPPRESSION_ENABLE_JSON_PTR,
        "/Equipment/CTRL/Settings/ADC/adc32_ch_suppress"
    );
}

#[test]
fn adc32_suppression_threshold_json_ptr() {
    assert_eq!(
        ADC32_SUPPRESSION_THRESHOLD_JSON_PTR,
        "/Equipment/CTRL/Settings/ADC/adc32_sthreshold"
    );
}

#[test]
fn bsc_pulser_enable_json_ptr() {
    assert_eq!(
        BSC_PULSER_ENABLE_JSON_PTR,
        "/Equipment/CTRL/Settings/BscPulserEnable"
    );
}

#[test]
fn field_wire_pulser_enable_json_ptr() {
    assert_eq!(
        FIELD_WIRE_PULSER_ENABLE_JSON_PTR,
        "/Equipment/CTRL/Settings/FwPulserEnable"
    );
}

#[test]
fn pulser_enable_json_ptr() {
    assert_eq!(
        PULSER_ENABLE_JSON_PTR,
        "/Equipment/CTRL/Settings/Pulser/Enable"
    );
}

#[test]
fn trigger_pulser_json_ptr() {
    assert_eq!(
        TRIGGER_PULSER_JSON_PTR,
        "/Equipment/CTRL/Settings/TrigSrc/TrigPulser"
    );
}

#[test]
fn pwb_force_channels_json_ptr() {
    assert_eq!(
        PWB_FORCE_CHANNELS_JSON_PTR,
        "/Equipment/CTRL/Settings/PWB/ch_force"
    );
}

#[test]
fn pwb_suppression_threshold_json_ptr() {
    assert_eq!(
        PWB_SUPPRESSION_THRESHOLD_JSON_PTR,
        "/Equipment/CTRL/Settings/PWB/ch_threshold"
    );
}

#[test]
fn trigger_sources_json_ptr() {
    assert_eq!(TRIGGER_SOURCES_JSON_PTR, "/Equipment/CTRL/Settings/TrigSrc");
}

#[test]
fn event_id_try_from_u16() {
    for num in 0..=u16::MAX {
        if num == 1 {
            assert!(matches!(EventId::try_from(num).unwrap(), EventId::Main));
        } else if num == 4 {
            assert!(matches!(
                EventId::try_from(num).unwrap(),
                EventId::Chronobox
            ));
        } else if num == 8 {
            assert!(matches!(
                EventId::try_from(num).unwrap(),
                EventId::Sequencer2
            ));
        } else {
            assert!(EventId::try_from(num).is_err());
        }
    }
}

#[test]
fn adc_16_bank_name_pattern_mismatch() {
    match Adc16BankName::try_from("C09A") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C09A");
        }
        _ => unreachable!(),
    }
    match Adc16BankName::try_from("B91") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B91");
        }
        _ => unreachable!(),
    }
    match Adc16BankName::try_from("B0911") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B0911");
        }
        _ => unreachable!(),
    }
    match Adc16BankName::try_from("B0¹") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B0¹");
        }
        _ => unreachable!(),
    }
    match Adc16BankName::try_from("B09 ") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B09 ");
        }
        _ => unreachable!(),
    }
    match Adc16BankName::try_from("B09a") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B09a");
        }
        _ => unreachable!(),
    }
    match Adc16BankName::try_from("b09A") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "b09A");
        }
        _ => unreachable!(),
    }
}

#[test]
fn adc_16_bank_name_unknown_board_id() {
    for num in 0..9 {
        let name = format!("B{num:0>2}0");
        assert!(matches!(
            Adc16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId(_))
        ));
    }
    for num in 19..100 {
        let name = format!("B{num}0");
        assert!(matches!(
            Adc16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId(_))
        ));
    }
}

#[test]
fn adc_16_bank_name_unknown_channel_id() {
    for chan in 'G'..='Z' {
        let name = format!("B09{chan}");
        assert!(matches!(
            Adc16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownChannelId(_))
        ));
    }
}

#[test]
fn valid_adc_16_bank_name() {
    for num in 9..=14 {
        for chan in 0..=9 {
            let bank_name = format!("B{num:0>2}{chan}");
            let bank_name = Adc16BankName::try_from(&bank_name[..]).unwrap();
            assert_eq!(
                bank_name.board_id,
                crate::alpha16::BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
            );
            assert_eq!(
                bank_name.channel_id,
                Adc16ChannelId::try_from(chan).unwrap()
            );
        }
    }
    for num in 9..=14 {
        for (i, chan) in ('A'..='F').enumerate() {
            let bank_name = format!("B{num:0>2}{chan}");
            let bank_name = Adc16BankName::try_from(&bank_name[..]).unwrap();
            assert_eq!(
                bank_name.board_id,
                crate::alpha16::BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
            );
            assert_eq!(
                bank_name.channel_id,
                Adc16ChannelId::try_from(u8::try_from(i).unwrap() + 10).unwrap()
            );
        }
    }
}

#[test]
fn adc_16_bank_name_board_id() {
    for num in 9..=14 {
        let bank_name = format!("B{num:0>2}0");
        let bank_name = Adc16BankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(
            bank_name.board_id(),
            crate::alpha16::BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
        );
    }
}

#[test]
fn adc_16_bank_name_channel_id() {
    for chan in 0..=9 {
        let bank_name = format!("B09{chan}");
        let bank_name = Adc16BankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(
            bank_name.channel_id(),
            Adc16ChannelId::try_from(chan).unwrap()
        );
    }
    for (i, chan) in ('A'..='F').enumerate() {
        let bank_name = format!("B09{chan}");
        let bank_name = Adc16BankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(
            bank_name.channel_id(),
            Adc16ChannelId::try_from(u8::try_from(i).unwrap() + 10).unwrap()
        );
    }
}

#[test]
fn adc_32_bank_name_pattern_mismatch() {
    match Adc32BankName::try_from("B09A") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B09A");
        }
        _ => unreachable!(),
    }
    match Adc32BankName::try_from("C91") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C91");
        }
        _ => unreachable!(),
    }
    match Adc32BankName::try_from("C0911") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C0911");
        }
        _ => unreachable!(),
    }
    match Adc32BankName::try_from("C0¹") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C0¹");
        }
        _ => unreachable!(),
    }
    match Adc32BankName::try_from("C09 ") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C09 ");
        }
        _ => unreachable!(),
    }
    match Adc32BankName::try_from("C09a") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C09a");
        }
        _ => unreachable!(),
    }
    match Adc32BankName::try_from("c09A") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "c09A");
        }
        _ => unreachable!(),
    }
}

#[test]
fn adc_32_bank_name_unknown_board_id() {
    for num in 0..9 {
        let name = format!("C{num:0>2}0");
        assert!(matches!(
            Adc32BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId(_))
        ));
    }
    for num in 19..100 {
        let name = format!("C{num}0");
        assert!(matches!(
            Adc32BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId(_))
        ));
    }
}

#[test]
fn adc_32_bank_name_unknown_channel_id() {
    for chan in 'W'..='Z' {
        let name = format!("C09{chan}");
        assert!(matches!(
            Adc32BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownChannelId(_))
        ));
    }
}

#[test]
fn valid_adc_32_bank_name() {
    for num in 9..=14 {
        for chan in 0..=9 {
            let bank_name = format!("C{num:0>2}{chan}");
            let bank_name = Adc32BankName::try_from(&bank_name[..]).unwrap();
            assert_eq!(
                bank_name.board_id,
                crate::alpha16::BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
            );
            assert_eq!(
                bank_name.channel_id,
                Adc32ChannelId::try_from(chan).unwrap()
            );
        }
    }
    for num in 9..=14 {
        for (i, chan) in ('A'..='V').enumerate() {
            let bank_name = format!("C{num:0>2}{chan}");
            let bank_name = Adc32BankName::try_from(&bank_name[..]).unwrap();
            assert_eq!(
                bank_name.board_id,
                crate::alpha16::BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
            );
            assert_eq!(
                bank_name.channel_id,
                Adc32ChannelId::try_from(u8::try_from(i).unwrap() + 10).unwrap()
            );
        }
    }
}

#[test]
fn adc_32_bank_name_board_id() {
    for num in 9..=14 {
        let bank_name = format!("C{num:0>2}0");
        let bank_name = Adc32BankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(
            bank_name.board_id(),
            crate::alpha16::BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
        );
    }
}

#[test]
fn adc_32_bank_name_channel_id() {
    for chan in 0..=9 {
        let bank_name = format!("C09{chan}");
        let bank_name = Adc32BankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(
            bank_name.channel_id(),
            Adc32ChannelId::try_from(chan).unwrap()
        );
    }
    for (i, chan) in ('A'..='V').enumerate() {
        let bank_name = format!("C09{chan}");
        let bank_name = Adc32BankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(
            bank_name.channel_id(),
            Adc32ChannelId::try_from(u8::try_from(i).unwrap() + 10).unwrap()
        );
    }
}

#[test]
fn alpha_16_bank_name_pattern_mismatch() {
    match Alpha16BankName::try_from("C91") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C91");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("C0911") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C0911");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("C0¹") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C0¹");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("C09 ") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C09 ");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("C09a") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "C09a");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("c09A") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "c09A");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("B91") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B91");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("B0911") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B0911");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("B0¹") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B0¹");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("B09 ") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B09 ");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("B09a") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "B09a");
        }
        _ => unreachable!(),
    }
    match Alpha16BankName::try_from("b09A") {
        Err(ParseAlpha16BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "b09A");
        }
        _ => unreachable!(),
    }
}

#[test]
fn alpha_16_bank_name_unknown_board_id() {
    for num in 0..9 {
        let name = format!("C{num:0>2}0");
        assert!(matches!(
            Alpha16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId(_))
        ));
        let name = format!("B{num:0>2}0");
        assert!(matches!(
            Alpha16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId(_))
        ));
    }
    for num in 19..100 {
        let name = format!("C{num}0");
        assert!(matches!(
            Alpha16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId(_))
        ));
        let name = format!("B{num}0");
        assert!(matches!(
            Alpha16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId(_))
        ));
    }
}

#[test]
fn alpha_16_bank_name_unknown_channel_id() {
    for chan in 'G'..='Z' {
        let name = format!("B09{chan}");
        assert!(matches!(
            Alpha16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownChannelId(_))
        ));
    }
    for chan in 'W'..='Z' {
        let name = format!("C09{chan}");
        assert!(matches!(
            Alpha16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownChannelId(_))
        ));
    }
}

#[test]
fn valid_alpha_16_bank_name() {
    for num in 9..=14 {
        for chan in 0..=9 {
            let bank_name = format!("B{num:0>2}{chan}");
            let bank_name = Alpha16BankName::try_from(&bank_name[..]).unwrap();
            assert!(matches!(bank_name, Alpha16BankName::A16(_)));
            let bank_name = format!("C{num:0>2}{chan}");
            let bank_name = Alpha16BankName::try_from(&bank_name[..]).unwrap();
            assert!(matches!(bank_name, Alpha16BankName::A32(_)));
        }
    }
    for num in 9..=14 {
        for chan in 'A'..='F' {
            let bank_name = format!("B{num:0>2}{chan}");
            let bank_name = Alpha16BankName::try_from(&bank_name[..]).unwrap();
            assert!(matches!(bank_name, Alpha16BankName::A16(_)));
        }
    }
    for num in 9..=14 {
        for chan in 'A'..='V' {
            let bank_name = format!("C{num:0>2}{chan}");
            let bank_name = Alpha16BankName::try_from(&bank_name[..]).unwrap();
            assert!(matches!(bank_name, Alpha16BankName::A32(_)));
        }
    }
}

#[test]
fn alpha_16_bank_name_board_id() {
    for num in 9..=14 {
        let board_id = format!("{num:0>2}");
        let board_id = crate::alpha16::BoardId::try_from(&board_id[..]).unwrap();

        let bank_name = format!("B{num:0>2}F");
        let bank_name = Alpha16BankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(bank_name.board_id(), board_id);
        let bank_name = format!("C{num:0>2}V");
        let bank_name = Alpha16BankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(bank_name.board_id(), board_id);
    }
}

#[test]
fn alpha_16_bank_name_channel_id() {
    for (i, chan) in ('0'..='9').chain('A'..='F').enumerate() {
        let bank_name = format!("B09{chan}");
        let bank_name = Alpha16BankName::try_from(&bank_name[..]).unwrap();
        match bank_name.channel_id() {
            ChannelId::A16(channel) => assert_eq!(
                channel,
                Adc16ChannelId::try_from(u8::try_from(i).unwrap()).unwrap()
            ),
            _ => unreachable!(),
        }
    }
    for (i, chan) in ('0'..='9').chain('A'..='V').enumerate() {
        let bank_name = format!("C09{chan}");
        let bank_name = Alpha16BankName::try_from(&bank_name[..]).unwrap();
        match bank_name.channel_id() {
            ChannelId::A32(channel) => assert_eq!(
                channel,
                Adc32ChannelId::try_from(u8::try_from(i).unwrap()).unwrap()
            ),
            _ => unreachable!(),
        }
    }
}

#[test]
fn padwing_bank_name_pattern_mismatch() {
    match PadwingBankName::try_from("pc00") {
        Err(ParsePadwingBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "pc00");
        }
        _ => unreachable!(),
    }
    match PadwingBankName::try_from("PC0") {
        Err(ParsePadwingBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "PC0");
        }
        _ => unreachable!(),
    }
    match PadwingBankName::try_from("PC000") {
        Err(ParsePadwingBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "PC000");
        }
        _ => unreachable!(),
    }
    match PadwingBankName::try_from("PC0 ") {
        Err(ParsePadwingBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "PC0 ");
        }
        _ => unreachable!(),
    }
    match PadwingBankName::try_from("PC0A") {
        Err(ParsePadwingBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "PC0A");
        }
        _ => unreachable!(),
    }
    match PadwingBankName::try_from("PC¹") {
        Err(ParsePadwingBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "PC¹");
        }
        _ => unreachable!(),
    }
}

#[test]
fn padwing_bank_name_unknown_board_id() {
    for num in 95..100 {
        let name = format!("PC{num}");
        assert!(matches!(
            PadwingBankName::try_from(&name[..]),
            Err(ParsePadwingBankNameError::UnknownBoardId(_))
        ));
    }
}

#[test]
fn padwing_bank_name_valid() {
    for num in 0..79 {
        if num == 9
            || num == 16
            || num == 28
            || num == 30
            || num == 31
            || num == 32
            || num == 38
            || num == 43
            || num == 47
            || num == 48
            || num == 50
            || num == 51
            || num == 59
            || num == 61
            || num == 62
        {
            continue;
        }
        let name = format!("PC{num:0>2}");
        let bank_name = PadwingBankName::try_from(&name[..]).unwrap();
        assert_eq!(
            bank_name.board_id(),
            crate::padwing::BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
        );
    }
}

#[test]
fn trigger_bank_name_pattern_mismatch() {
    match TriggerBankName::try_from("atat") {
        Err(ParseTriggerBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "atat");
        }
        _ => unreachable!(),
    }
    match TriggerBankName::try_from("Atat") {
        Err(ParseTriggerBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "Atat");
        }
        _ => unreachable!(),
    }
    match TriggerBankName::try_from("aTat") {
        Err(ParseTriggerBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "aTat");
        }
        _ => unreachable!(),
    }
    match TriggerBankName::try_from("atAt") {
        Err(ParseTriggerBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "atAt");
        }
        _ => unreachable!(),
    }
    match TriggerBankName::try_from("ataT") {
        Err(ParseTriggerBankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "ataT");
        }
        _ => unreachable!(),
    }
}

#[test]
fn trigger_bank_name_valid() {
    assert!(TriggerBankName::try_from("ATAT").is_ok());
}

#[test]
fn trb3_bank_name_pattern_mismatch() {
    match Trb3BankName::try_from("trba") {
        Err(ParseTrb3BankNameError::PatternMismatch { input }) => {
            assert_eq!(input, "trba");
        }
        _ => unreachable!(),
    }
}

#[test]
fn trb3_bank_name_valid() {
    assert!(Trb3BankName::try_from("TRBA").is_ok());
}

#[test]
fn mc_vertex_bank_name_pattern_mismatch() {
    assert!(McVertexBankName::try_from("mcvx").is_err());
    assert!(McVertexBankName::try_from("Mcvx").is_err());
}

#[test]
fn seq2_bank_name_valid() {
    assert!(Seq2BankName::try_from("SEQ2").is_ok());
}

#[test]
fn seq2_bank_name_pattern_mismatch() {
    assert!(Seq2BankName::try_from("seq2").is_err());
    assert!(Seq2BankName::try_from("Seq2").is_err());
}

#[test]
fn mc_vertex_bank_name_valid() {
    assert!(McVertexBankName::try_from("MCVX").is_ok());
}

#[test]
fn main_event_bank_name_invalid() {
    assert!(MainEventBankName::try_from("atat").is_err());
    assert!(MainEventBankName::try_from("Atat").is_err());
    assert!(MainEventBankName::try_from("B09").is_err());
    assert!(MainEventBankName::try_from("C09").is_err());
    assert!(MainEventBankName::try_from("PCXX").is_err());
    assert!(MainEventBankName::try_from("Trba").is_err());
    assert!(MainEventBankName::try_from("Mcvx").is_err());
}

#[test]
fn main_event_bank_name_valid() {
    assert!(matches!(
        MainEventBankName::try_from("ATAT").unwrap(),
        MainEventBankName::Trg(_)
    ));
    assert!(matches!(
        MainEventBankName::try_from("B09A").unwrap(),
        MainEventBankName::Alpha16(_)
    ));
    assert!(matches!(
        MainEventBankName::try_from("C09A").unwrap(),
        MainEventBankName::Alpha16(_)
    ));
    assert!(matches!(
        MainEventBankName::try_from("PC00").unwrap(),
        MainEventBankName::Padwing(_)
    ));
    assert!(matches!(
        MainEventBankName::try_from("TRBA").unwrap(),
        MainEventBankName::Trb3(_)
    ));
    assert!(matches!(
        MainEventBankName::try_from("MCVX").unwrap(),
        MainEventBankName::McVertex(_)
    ));
}

#[test]
fn chronobox_bank_name_valid() {
    assert_eq!(
        ChronoboxBankName::try_from("CBF1").unwrap(),
        ChronoboxBankName {
            board_id: crate::chronobox::BoardId::try_from("cb01").unwrap()
        }
    );
    assert_eq!(
        ChronoboxBankName::try_from("CBF2").unwrap(),
        ChronoboxBankName {
            board_id: crate::chronobox::BoardId::try_from("cb02").unwrap()
        }
    );
    assert_eq!(
        ChronoboxBankName::try_from("CBF3").unwrap(),
        ChronoboxBankName {
            board_id: crate::chronobox::BoardId::try_from("cb03").unwrap()
        }
    );
    assert_eq!(
        ChronoboxBankName::try_from("CBF4").unwrap(),
        ChronoboxBankName {
            board_id: crate::chronobox::BoardId::try_from("cb04").unwrap()
        }
    );
}

#[test]
fn chronobox_bank_name_invalid() {
    assert!(ChronoboxBankName::try_from("CBFT").is_err());
    assert!(ChronoboxBankName::try_from("CBF0").is_err());
    assert!(ChronoboxBankName::try_from("CBF5").is_err());
}
