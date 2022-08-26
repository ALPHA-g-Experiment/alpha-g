use super::*;
use alpha_g_detector::padwing::{ChannelId, PwbPacket};

const ODD_PWB_V2_PACKET: [u8; 120] = [
    2, 68, 0, 0, 236, 40, 255, 135, 84, 2, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 5, 0, 6, 7, 49, 0, 5, 0, 255, 7, 3, 0,
    5, 0, 7, 0, 9, 0, 0, 0, 57, 0, 5, 0, 0, 248, 13, 0, 15, 0, 17, 0, 19, 0, 0, 0, 65, 0, 5, 0,
    255, 7, 0, 248, 25, 0, 27, 0, 29, 0, 0, 0, 73, 0, 5, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0,
    204, 204, 204, 204,
];

#[test]
fn packet_overflow() {
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(49).unwrap(),
    };
    assert_eq!(packet.overflow(), Overflow::Positive);

    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(57).unwrap(),
    };
    assert_eq!(packet.overflow(), Overflow::Negative);

    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(65).unwrap(),
    };
    assert_eq!(packet.overflow(), Overflow::Both);

    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(73).unwrap(),
    };
    assert_eq!(packet.overflow(), Overflow::Neither);
}

#[test]
fn packet_passes_overflow_filter() {
    let filter = Filter {
        overflow: Some(Overflow::Positive),
        ..Filter::default()
    };
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(49).unwrap(),
    };
    assert!(packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(57).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(65).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(73).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));

    let filter = Filter {
        overflow: Some(Overflow::Negative),
        ..Filter::default()
    };
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(49).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(57).unwrap(),
    };
    assert!(packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(65).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(73).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));

    let filter = Filter {
        overflow: Some(Overflow::Both),
        ..Filter::default()
    };
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(49).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(57).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(65).unwrap(),
    };
    assert!(packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(73).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));

    let filter = Filter {
        overflow: Some(Overflow::Neither),
        ..Filter::default()
    };
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(49).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(57).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(65).unwrap(),
    };
    assert!(!packet.passes_filter(&filter));
    let packet = Packet {
        pwb_packet: PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap(),
        channel_id: ChannelId::try_from(73).unwrap(),
    };
    assert!(packet.passes_filter(&filter));
}
