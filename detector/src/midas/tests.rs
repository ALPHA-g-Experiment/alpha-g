use super::*;

#[test]
fn event_id_try_from_u16() {
    for num in 0..=u16::MAX {
        if num == 1 {
            assert!(matches!(EventId::try_from(num).unwrap(), EventId::Main));
        } else {
            assert!(EventId::try_from(num).is_err());
        }
    }
}

#[test]
fn adc_16_bank_name_pattern_mismatch() {
    assert!(matches!(
        Adc16BankName::try_from("C09A"),
        Err(ParseAlpha16BankNameError::PatternMismatch)
    ));
    assert!(matches!(
        Adc16BankName::try_from("B91"),
        Err(ParseAlpha16BankNameError::PatternMismatch)
    ));
    assert!(matches!(
        Adc16BankName::try_from("B0911"),
        Err(ParseAlpha16BankNameError::PatternMismatch)
    ));
    assert!(matches!(
        Adc16BankName::try_from("B0¹"),
        Err(ParseAlpha16BankNameError::PatternMismatch)
    ));
    assert!(matches!(
        Adc16BankName::try_from("B09 "),
        Err(ParseAlpha16BankNameError::PatternMismatch)
    ));
    assert!(matches!(
        Adc16BankName::try_from("B09a"),
        Err(ParseAlpha16BankNameError::PatternMismatch)
    ));
    assert!(matches!(
        Adc16BankName::try_from("b09A"),
        Err(ParseAlpha16BankNameError::PatternMismatch)
    ));
}

#[test]
fn adc_16_bank_name_unknown_board_id() {
    for num in 0..9 {
        let name = format!("B{num:0>2}0");
        assert!(matches!(
            Adc16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId)
        ));
    }
    for num in 19..100 {
        let name = format!("B{num}0");
        assert!(matches!(
            Adc16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownBoardId)
        ));
    }
}

#[test]
fn adc_16_bank_name_unknown_channel_id() {
    for chan in 'G'..='Z' {
        let name = format!("B09{chan}");
        assert!(matches!(
            Adc16BankName::try_from(&name[..]),
            Err(ParseAlpha16BankNameError::UnknownChannelId)
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
                BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
            );
            assert_eq!(
                bank_name.channel_id,
                Adc16ChannelId::try_from(chan).unwrap()
            );
        }
    }
    for num in 9..=14 {
        for (i, chan) in ('A'..='F').into_iter().enumerate() {
            let bank_name = format!("B{num:0>2}{chan}");
            let bank_name = Adc16BankName::try_from(&bank_name[..]).unwrap();
            assert_eq!(
                bank_name.board_id,
                BoardId::try_from(&format!("{num:0>2}")[..]).unwrap()
            );
            assert_eq!(
                bank_name.channel_id,
                Adc16ChannelId::try_from(u8::try_from(i).unwrap() + 10).unwrap()
            );
        }
    }
}