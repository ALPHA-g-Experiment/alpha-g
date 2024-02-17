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
