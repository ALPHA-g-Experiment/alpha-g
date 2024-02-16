use super::*;

#[test]
fn try_chronobox_channel_id() {
    for num in 0..=59 {
        assert_eq!(ChannelId(num), ChannelId::try_from(num).unwrap());
    }
    for num in 60..=255 {
        assert!(ChannelId::try_from(num).is_err());
    }
}
