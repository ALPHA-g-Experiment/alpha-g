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
        for (i, chan) in ('A'..='F').into_iter().enumerate() {
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
    for (i, chan) in ('A'..='F').into_iter().enumerate() {
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
        for (i, chan) in ('A'..='V').into_iter().enumerate() {
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
    for (i, chan) in ('A'..='V').into_iter().enumerate() {
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
    for (i, chan) in ('0'..='9')
        .into_iter()
        .chain(('A'..='F').into_iter())
        .enumerate()
    {
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
    for (i, chan) in ('0'..='9')
        .into_iter()
        .chain(('A'..='V').into_iter())
        .enumerate()
    {
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
