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
