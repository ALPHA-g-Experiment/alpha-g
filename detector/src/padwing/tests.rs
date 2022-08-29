use super::*;
use crate::midas::PadwingBankName;

#[test]
fn padwing_rate() {
    assert_eq!(PADWING_RATE, 62.5e6);
}

#[test]
fn padwing_boards() {
    for i in 0..PADWINGBOARDS.len() {
        let next = i + 1;
        for j in next..PADWINGBOARDS.len() {
            assert_ne!(PADWINGBOARDS[i].0, PADWINGBOARDS[j].0);
            assert_ne!(PADWINGBOARDS[i].1, PADWINGBOARDS[j].1);
            assert_ne!(PADWINGBOARDS[i].2, PADWINGBOARDS[j].2);
        }
    }
}

#[test]
fn bank_names_from_padwing_boards() {
    for triplet in PADWINGBOARDS {
        let bank_name = format!("PC{}", triplet.0);
        let bank_name = PadwingBankName::try_from(&bank_name[..]).unwrap();
        assert_eq!(bank_name.board_id(), BoardId::try_from(triplet.1).unwrap());
    }
}

#[test]
fn board_id() {
    for triplet in PADWINGBOARDS {
        let from_name = BoardId::try_from(triplet.0).unwrap();
        let from_mac = BoardId::try_from(triplet.1).unwrap();
        let from_id = BoardId::try_from(triplet.2).unwrap();

        assert_eq!(from_name, from_mac);
        assert_eq!(from_mac, from_id);

        assert_eq!(from_id.name(), triplet.0);
        assert_eq!(from_id.mac_address(), triplet.1);
        assert_eq!(from_id.device_id(), triplet.2);
    }
}

#[test]
fn try_from_unsigned_after() {
    assert!(matches!(AfterId::try_from(0).unwrap(), AfterId::A));
    assert!(matches!(AfterId::try_from(1).unwrap(), AfterId::B));
    assert!(matches!(AfterId::try_from(2).unwrap(), AfterId::C));
    assert!(matches!(AfterId::try_from(3).unwrap(), AfterId::D));

    for i in 4..u8::MAX {
        assert!(AfterId::try_from(i).is_err());
    }
}

#[test]
fn try_from_char_after() {
    assert!(matches!(AfterId::try_from('A').unwrap(), AfterId::A));
    assert!(matches!(AfterId::try_from('B').unwrap(), AfterId::B));
    assert!(matches!(AfterId::try_from('C').unwrap(), AfterId::C));
    assert!(matches!(AfterId::try_from('D').unwrap(), AfterId::D));

    for i in 'a'..'z' {
        assert!(AfterId::try_from(i).is_err());
    }
    for i in 'E'..'Z' {
        assert!(AfterId::try_from(i).is_err());
    }
}

const CHUNK: [u8; 28] = [
    236, 40, 255, 135, 2, 0, 0, 0, 3, 0, 0, 1, 5, 0, 1, 0, 143, 203, 131, 81, 255, 0, 0, 0, 122,
    92, 155, 159,
];

#[test]
fn chunk_to_string() {
    let chunk = Chunk::try_from(&CHUNK[..]).unwrap();
    assert_eq!(
        format!("{chunk}"),
        "Device ID: 2281646316
Packet sequence: 2
Channel sequence: 3
Channel ID: 0
Flags: 00000001
Chunk ID: 5
Chunk length: 1
Header CRC-32C: 1367591823
Payload CRC-32C: 2677759098"
    );
}

#[test]
fn chunk_good() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
        let crc = !crc32c::crc32c(&good_chunk[0..16]);
        good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
        assert!(Chunk::try_from(&good_chunk[..]).is_ok());
    }

    let mut good_chunk = CHUNK;
    for num in 0..4 {
        good_chunk[10] = num;
        let crc = !crc32c::crc32c(&good_chunk[0..16]);
        good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
        assert!(Chunk::try_from(&good_chunk[..]).is_ok());
    }
}

#[test]
fn chunk_incomplete_slice() {
    let mut bad_chunk: Vec<u8> = CHUNK.to_vec();
    bad_chunk.insert(20, 0);
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::IncompleteSlice {
            found,
            min_expected,
        }) => {
            assert_eq!(found, 29);
            assert_eq!(min_expected, 32);
        }
        _ => unreachable!(),
    }

    let bad_chunk = &CHUNK[..10];
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::IncompleteSlice {
            found,
            min_expected,
        }) => {
            assert_eq!(found, 10);
            assert_eq!(min_expected, 28);
        }
        _ => unreachable!(),
    }
}

#[test]
fn chunk_unknown_device_id() {
    for num in 0..u16::MAX {
        let mut bad_chunk = CHUNK;
        bad_chunk[0..4].copy_from_slice(&u32::from(num).to_le_bytes()[..]);
        assert!(matches!(
            Chunk::try_from(&bad_chunk[..]),
            Err(TryChunkFromSliceError::UnknownDeviceId(_))
        ));
    }
}

#[test]
fn chunk_unknown_channel_id() {
    for num in 4..u8::MAX {
        let mut bad_chunk = CHUNK;
        bad_chunk[10] = num;
        assert!(matches!(
            Chunk::try_from(&bad_chunk[..]),
            Err(TryChunkFromSliceError::UnknownChannelId(_))
        ));
    }
}

#[test]
fn chunk_unknown_flags() {
    for num in 2..u8::MAX {
        let mut bad_chunk = CHUNK;
        bad_chunk[11] = num;
        match Chunk::try_from(&bad_chunk[..]) {
            Err(TryChunkFromSliceError::UnknownFlags { found }) => {
                assert_eq!(found, num);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn chunk_bad_chunk_length() {
    let mut bad_chunk = CHUNK;
    bad_chunk[14] = 0;
    let crc = !crc32c::crc32c(&bad_chunk[0..16]);
    bad_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::BadChunkLength { found, min, max }) => {
            assert_eq!(found, 0);
            assert_eq!(min, 1);
            assert_eq!(max, 4);
        }
        _ => unreachable!(),
    }

    bad_chunk[14] = 5;
    let crc = !crc32c::crc32c(&bad_chunk[0..16]);
    bad_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::BadChunkLength { found, min, max }) => {
            assert_eq!(found, 5);
            assert_eq!(min, 1);
            assert_eq!(max, 4);
        }
        _ => unreachable!(),
    }
}

#[test]
fn chunk_zero_mismatch() {
    let mut bad_chunk = CHUNK;
    bad_chunk[21] = 1;
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, vec![1, 0, 0]);
        }
        _ => unreachable!(),
    }

    let mut bad_chunk = CHUNK;
    bad_chunk[22] = 1;
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, vec![0, 1, 0]);
        }
        _ => unreachable!(),
    }

    let mut bad_chunk = CHUNK;
    bad_chunk[23] = 1;
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::ZeroMismatch { found }) => {
            assert_eq!(found, vec![0, 0, 1]);
        }
        _ => unreachable!(),
    }
}

#[test]
fn chunk_header_crc_mismatch() {
    let mut bad_chunk = CHUNK;
    bad_chunk[16..20].copy_from_slice(&[0, 0, 0, 0]);
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::HeaderCRC32CMismatch { found, expected }) => {
            assert_eq!(found, 0);
            assert_eq!(expected, 1367591823);
        }
        _ => unreachable!(),
    }
}

#[test]
fn chunk_payload_crc_mismatch() {
    let mut bad_chunk = CHUNK;
    bad_chunk[24..28].copy_from_slice(&[0, 0, 0, 0]);
    match Chunk::try_from(&bad_chunk[..]) {
        Err(TryChunkFromSliceError::PayloadCRC32CMismatch { found, expected }) => {
            assert_eq!(found, 0);
            assert_eq!(expected, 2677759098);
        }
        _ => unreachable!(),
    }
}

#[test]
fn chunk_board_id() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert_eq!(
                Chunk::try_from(&good_chunk[..]).unwrap().board_id(),
                BoardId::try_from(triplet.0).unwrap()
            );
        }
    }
}

#[test]
fn chunk_packet_sequence() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert_eq!(
                Chunk::try_from(&good_chunk[..]).unwrap().packet_sequence(),
                2
            );
        }
    }
}

#[test]
fn chunk_channel_sequence() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert_eq!(
                Chunk::try_from(&good_chunk[..]).unwrap().channel_sequence(),
                3
            );
        }
    }
}

#[test]
fn chunk_after_id() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert_eq!(
                Chunk::try_from(&good_chunk[..]).unwrap().after_id(),
                AfterId::try_from(num).unwrap()
            );
        }
    }
}

#[test]
fn chunk_is_end_of_message() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert!(Chunk::try_from(&good_chunk[..])
                .unwrap()
                .is_end_of_message());
        }
    }

    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            good_chunk[11] = 0;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert!(!Chunk::try_from(&good_chunk[..])
                .unwrap()
                .is_end_of_message());
        }
    }
}

#[test]
fn chunk_chunk_id() {
    let mut good_chunk = CHUNK;
    for id in 0..u16::MAX {
        good_chunk[12..14].copy_from_slice(&id.to_le_bytes()[..]);
        let crc = !crc32c::crc32c(&good_chunk[0..16]);
        good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
        assert_eq!(Chunk::try_from(&good_chunk[..]).unwrap().chunk_id(), id);
    }
}

#[test]
fn chunk_header_crc() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert_eq!(
                Chunk::try_from(&good_chunk[..]).unwrap().header_crc32c(),
                crc
            );
        }
    }
}

#[test]
fn chunk_payload() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert_eq!(Chunk::try_from(&good_chunk[..]).unwrap().payload(), [255]);
        }
    }
}

#[test]
fn chunk_payload_crc() {
    let mut good_chunk = CHUNK;
    for triplet in PADWINGBOARDS {
        for num in 0..4 {
            good_chunk[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
            good_chunk[10] = num;
            let crc = !crc32c::crc32c(&good_chunk[0..16]);
            good_chunk[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
            assert_eq!(
                Chunk::try_from(&good_chunk[..]).unwrap().payload_crc32c(),
                2677759098
            );
        }
    }
}

#[test]
fn try_from_unsigned_compression() {
    assert!(matches!(
        Compression::try_from(0).unwrap(),
        Compression::Raw
    ));

    for i in 1..u8::MAX {
        assert!(Compression::try_from(i).is_err());
    }
}

#[test]
fn try_from_unsigned_trigger() {
    assert!(matches!(Trigger::try_from(0).unwrap(), Trigger::External));
    assert!(matches!(Trigger::try_from(1).unwrap(), Trigger::Manual));
    assert!(matches!(
        Trigger::try_from(3).unwrap(),
        Trigger::InternalPulse
    ));

    assert!(Trigger::try_from(2).is_err());
    for i in 4..u8::MAX {
        assert!(Trigger::try_from(i).is_err());
    }
}

#[test]
fn try_from_unsigned_channel_id() {
    assert!(ChannelId::try_from(0).is_err());
    for i in 1..=3 {
        assert!(matches!(
            ChannelId::try_from(i).unwrap(),
            ChannelId::Reset(_)
        ));
    }
    for i in 4..=15 {
        assert!(matches!(ChannelId::try_from(i).unwrap(), ChannelId::Pad(_)));
    }
    assert!(matches!(
        ChannelId::try_from(16).unwrap(),
        ChannelId::Fpn(_)
    ));
    for i in 17..=28 {
        assert!(matches!(ChannelId::try_from(i).unwrap(), ChannelId::Pad(_)));
    }
    assert!(matches!(
        ChannelId::try_from(29).unwrap(),
        ChannelId::Fpn(_)
    ));
    for i in 30..=53 {
        assert!(matches!(ChannelId::try_from(i).unwrap(), ChannelId::Pad(_)));
    }
    assert!(matches!(
        ChannelId::try_from(54).unwrap(),
        ChannelId::Fpn(_)
    ));
    for i in 55..=66 {
        assert!(matches!(ChannelId::try_from(i).unwrap(), ChannelId::Pad(_)));
    }
    assert!(matches!(
        ChannelId::try_from(67).unwrap(),
        ChannelId::Fpn(_)
    ));
    for i in 68..=79 {
        assert!(matches!(ChannelId::try_from(i).unwrap(), ChannelId::Pad(_)));
    }
    assert!(ChannelId::try_from(80).is_err());
}

const ODD_PWB_V2_PACKET: [u8; 104] = [
    2, 68, 0, 0, 236, 40, 255, 135, 84, 2, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0,
    0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 5, 0, 6, 7, 57, 0, 5, 0, 1, 2, 3, 4,
    5, 6, 7, 8, 9, 10, 0, 0, 65, 0, 5, 0, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 0, 0, 73, 0, 5,
    0, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 0, 0, 204, 204, 204, 204,
];

#[test]
fn pwb_v2_packet_to_string() {
    let packet = PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..]).unwrap();
    assert_eq!(
        format!("{packet}"),
        "Format revision: 2
AFTER ID: D
Compression: Raw
Trigger source: External
MAC address: [236, 40, 255, 135, 84, 2]
Trigger delay: 1
Trigger timestamp: 2
Last SCA cell: 3
Requested samples: 5
Channels sent: [57, 65, 73]
Channels over threshold: [1, 9, 17]
Event counter: 4
FIFO max depth: 5
Event descriptor write depth: 6
Event descriptor read depth: 7"
    );
}

#[test]
fn pwb_v2_good() {
    let mut good_packet = ODD_PWB_V2_PACKET;
    for i in 65..=68 {
        good_packet[1] = i;
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in [0, 1, 3] {
        good_packet[3] = i;
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for triplet in PADWINGBOARDS {
        good_packet[4..10].copy_from_slice(&triplet.1[..]);
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 0..=u8::MAX {
        good_packet[10] = i;
        good_packet[11] = i;
        good_packet[12] = i;
        good_packet[13] = i;
        good_packet[14] = i;
        good_packet[15] = i;
        good_packet[16] = i;
        good_packet[17] = i;
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 0..=511u16 {
        let i = i.to_le_bytes();
        good_packet[20..22].copy_from_slice(&i[..]);
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 0..=u8::MAX {
        good_packet[44] = i;
        good_packet[45] = i;
        good_packet[46] = i;
        good_packet[47] = i;
        good_packet[48] = i;
        good_packet[49] = i;
        good_packet[50] = i;
        good_packet[51] = i;
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    // Even packet
    for i in [22, 54, 70, 86] {
        good_packet[i] = 6;
    }
    assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    for i in 65..=68 {
        good_packet[1] = i;
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in [0, 1, 3] {
        good_packet[3] = i;
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for triplet in PADWINGBOARDS {
        good_packet[4..10].copy_from_slice(&triplet.1[..]);
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 0..=u8::MAX {
        good_packet[10] = i;
        good_packet[11] = i;
        good_packet[12] = i;
        good_packet[13] = i;
        good_packet[14] = i;
        good_packet[15] = i;
        good_packet[16] = i;
        good_packet[17] = i;
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 0..=511u16 {
        let i = i.to_le_bytes();
        good_packet[20..22].copy_from_slice(&i[..]);
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
    for i in 0..=u8::MAX {
        good_packet[44] = i;
        good_packet[45] = i;
        good_packet[46] = i;
        good_packet[47] = i;
        good_packet[48] = i;
        good_packet[49] = i;
        good_packet[50] = i;
        good_packet[51] = i;
        assert!(PwbV2Packet::try_from(&good_packet[..]).is_ok());
    }
}

#[test]
fn pwb_v2_packet_incomplete_slice() {
    let mut bad_packet: Vec<u8> = ODD_PWB_V2_PACKET.to_vec();
    bad_packet.push(0);
    match PwbV2Packet::try_from(&bad_packet[..]) {
        Err(TryPwbPacketFromSliceError::IncompleteSlice {
            found,
            min_expected,
        }) => {
            assert_eq!(found, 105);
            assert_eq!(min_expected, 104);
        }
        _ => unreachable!(),
    }

    let bad_packet = &ODD_PWB_V2_PACKET[..10];
    match PwbV2Packet::try_from(&bad_packet[..]) {
        Err(TryPwbPacketFromSliceError::IncompleteSlice {
            found,
            min_expected,
        }) => {
            assert_eq!(found, 10);
            assert_eq!(min_expected, 56);
        }
        _ => unreachable!(),
    }
}

#[test]
fn pwb_v2_packet_unknown_version() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    for i in 0..=u8::MAX {
        if i == 2 {
            continue;
        }
        bad_packet[0] = i;
        match PwbV2Packet::try_from(&bad_packet[..]) {
            Err(TryPwbPacketFromSliceError::UnknownVersion { found }) => {
                assert_eq!(found, i);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn pwb_v2_packet_unknown_after_id() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    for i in 0..=u8::MAX {
        if i >= 65 && i <= 68 {
            continue;
        }
        bad_packet[1] = i;
        assert!(matches!(
            PwbV2Packet::try_from(&bad_packet[..]),
            Err(TryPwbPacketFromSliceError::UnknownAfterId(_))
        ));
    }
}

#[test]
fn pwb_v2_packet_unknown_compression() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    for i in 1..=u8::MAX {
        bad_packet[2] = i;
        assert!(matches!(
            PwbV2Packet::try_from(&bad_packet[..]),
            Err(TryPwbPacketFromSliceError::UnknownCompression(_))
        ));
    }
}

#[test]
fn pwb_v2_packet_unknown_trigger() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    for i in 1..=u8::MAX {
        if i == 0 || i == 1 || i == 3 {
            continue;
        }
        bad_packet[3] = i;
        assert!(matches!(
            PwbV2Packet::try_from(&bad_packet[..]),
            Err(TryPwbPacketFromSliceError::UnknownTrigger(_))
        ));
    }
}

#[test]
fn pwb_v2_packet_unknown_mac() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    bad_packet[4..10].copy_from_slice(&[0, 0, 0, 0, 0, 0]);
    assert!(matches!(
        PwbV2Packet::try_from(&bad_packet[..]),
        Err(TryPwbPacketFromSliceError::UnknownMac(_))
    ));
}

#[test]
fn pwb_v2_packet_zero_mismatch() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    for i in 1..=u8::MAX {
        bad_packet[18] = i;
        match PwbV2Packet::try_from(&bad_packet[..]) {
            Err(TryPwbPacketFromSliceError::ZeroMismatch { found }) => {
                assert_eq!(found, [i, 0]);
            }
            _ => unreachable!(),
        }
    }

    let mut bad_packet = ODD_PWB_V2_PACKET;
    for i in 1..=u8::MAX {
        bad_packet[19] = i;
        match PwbV2Packet::try_from(&bad_packet[..]) {
            Err(TryPwbPacketFromSliceError::ZeroMismatch { found }) => {
                assert_eq!(found, [0, i]);
            }
            _ => unreachable!(),
        }
    }

    for i in [66, 82, 98] {
        let mut bad_packet = ODD_PWB_V2_PACKET;
        for j in 1..=u8::MAX {
            bad_packet[i] = j;
            match PwbV2Packet::try_from(&bad_packet[..]) {
                Err(TryPwbPacketFromSliceError::ZeroMismatch { found }) => {
                    assert_eq!(found, [j, 0]);
                }
                _ => unreachable!(),
            }
        }
    }

    for i in [67, 83, 99] {
        let mut bad_packet = ODD_PWB_V2_PACKET;
        for j in 1..=u8::MAX {
            bad_packet[i] = j;
            match PwbV2Packet::try_from(&bad_packet[..]) {
                Err(TryPwbPacketFromSliceError::ZeroMismatch { found }) => {
                    assert_eq!(found, [0, j]);
                }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn pwb_v2_packet_bad_last_sca_cell() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    let i = 512u16.to_le_bytes();
    bad_packet[20..22].copy_from_slice(&i[..]);
    match PwbV2Packet::try_from(&bad_packet[..]) {
        Err(TryPwbPacketFromSliceError::BadLastScaCell { found }) => {
            assert_eq!(found, 512);
        }
        _ => unreachable!(),
    }
}

#[test]
fn pwb_v2_packet_bad_sca_samples() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    let i = 512u16.to_le_bytes();
    bad_packet[22..24].copy_from_slice(&i[..]);
    match PwbV2Packet::try_from(&bad_packet[..]) {
        Err(TryPwbPacketFromSliceError::BadScaSamples { found }) => {
            assert_eq!(found, 512);
        }
        _ => unreachable!(),
    }
}

#[test]
fn pwb_v2_packet_bad_sca_channels_sent() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    for mut i in 1..=u8::MAX {
        i |= 128;
        bad_packet[33] = i;
        assert!(matches!(
            PwbV2Packet::try_from(&bad_packet[..]),
            Err(TryPwbPacketFromSliceError::BadScaChannelsSent)
        ));
    }
}

#[test]
fn pwb_v2_packet_bad_sca_channels_threshold() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    for mut i in 1..=u8::MAX {
        i |= 128;
        bad_packet[43] = i;
        assert!(matches!(
            PwbV2Packet::try_from(&bad_packet[..]),
            Err(TryPwbPacketFromSliceError::BadScaChannelsThreshold)
        ));
    }
}

#[test]
fn pwb_v2_packet_unknown_channel_id() {
    for j in [52, 68, 84] {
        let mut bad_packet = ODD_PWB_V2_PACKET;
        for i in 80..=u8::MAX {
            bad_packet[j] = i;
            assert!(matches!(
                PwbV2Packet::try_from(&bad_packet[..]),
                Err(TryPwbPacketFromSliceError::UnknownChannelId(_))
            ));
        }
    }
}

#[test]
fn pwb_v2_packet_channel_id_mismatch() {
    let mut bad_packet = ODD_PWB_V2_PACKET;
    bad_packet[52] = 58;
    match PwbV2Packet::try_from(&bad_packet[..]) {
        Err(TryPwbPacketFromSliceError::ChannelIdMismatch { found, expected }) => {
            assert_eq!(found, ChannelId::try_from(58).unwrap());
            assert_eq!(expected, ChannelId::try_from(57).unwrap());
        }
        _ => unreachable!(),
    }

    let mut bad_packet = ODD_PWB_V2_PACKET;
    bad_packet[68] = 66;
    match PwbV2Packet::try_from(&bad_packet[..]) {
        Err(TryPwbPacketFromSliceError::ChannelIdMismatch { found, expected }) => {
            assert_eq!(found, ChannelId::try_from(66).unwrap());
            assert_eq!(expected, ChannelId::try_from(65).unwrap());
        }
        _ => unreachable!(),
    }

    let mut bad_packet = ODD_PWB_V2_PACKET;
    bad_packet[84] = 74;
    match PwbV2Packet::try_from(&bad_packet[..]) {
        Err(TryPwbPacketFromSliceError::ChannelIdMismatch { found, expected }) => {
            assert_eq!(found, ChannelId::try_from(74).unwrap());
            assert_eq!(expected, ChannelId::try_from(73).unwrap());
        }
        _ => unreachable!(),
    }
}

#[test]
fn pwb_v2_packet_number_of_samples_mismatch() {
    for i in [54, 70, 86] {
        let mut bad_packet = ODD_PWB_V2_PACKET;
        bad_packet[i] = 6;
        match PwbV2Packet::try_from(&bad_packet[..]) {
            Err(TryPwbPacketFromSliceError::NumberOfSamplesMismatch { found, expected }) => {
                assert_eq!(found, 6);
                assert_eq!(expected, 5);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn pwb_v2_packet_bad_end_of_data_marker() {
    for i in 100..=103 {
        let mut bad_packet = ODD_PWB_V2_PACKET;
        bad_packet[i] = 0;
        match PwbV2Packet::try_from(&bad_packet[..]) {
            Err(TryPwbPacketFromSliceError::BadEndOfDataMarker { found }) => {
                let mut eod_marker = [204, 204, 204, 204];
                eod_marker[i - 100] = 0;
                assert_eq!(found, eod_marker);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn pwb_v2_packet_version() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .packet_version(),
        2
    );
}

#[test]
fn pwb_v2_packet_after_id() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .after_id(),
        AfterId::try_from('D').unwrap()
    );
}

#[test]
fn pwb_v2_packet_compression() {
    assert!(matches!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .compression(),
        Compression::Raw
    ));
}

#[test]
fn pwb_v2_packet_trigger_source() {
    assert!(matches!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .trigger_source(),
        Trigger::External
    ));
}

#[test]
fn pwb_v2_packet_board_id() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .board_id(),
        BoardId::try_from("00").unwrap()
    );
}

#[test]
fn pwb_v2_packet_trigger_delay() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .trigger_delay(),
        1
    );
}

#[test]
fn pwb_v2_packet_trigger_timestamp() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .trigger_timestamp(),
        2
    );
}

#[test]
fn pwb_v2_packet_last_sca_cell() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .last_sca_cell(),
        3
    );
}

#[test]
fn pwb_v2_packet_requested_samples() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .requested_samples(),
        5
    );
}

#[test]
fn pwb_v2_packet_channels_sent() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .channels_sent(),
        [
            ChannelId::try_from(57).unwrap(),
            ChannelId::try_from(65).unwrap(),
            ChannelId::try_from(73).unwrap()
        ]
    );
}

#[test]
fn pwb_v2_packet_channels_over_threshold() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .channels_over_threshold(),
        [
            ChannelId::try_from(1).unwrap(),
            ChannelId::try_from(9).unwrap(),
            ChannelId::try_from(17).unwrap()
        ]
    );
}

#[test]
fn pwb_v2_packet_event_counter() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .event_counter(),
        4
    );
}

#[test]
fn pwb_v2_packet_fifo_max_depth() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .fifo_max_depth(),
        5
    );
}

#[test]
fn pwb_v2_packet_event_descriptor_write_depth() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .event_descriptor_write_depth(),
        6
    );
}

#[test]
fn pwb_v2_packet_event_descriptor_read_depth() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .event_descriptor_read_depth(),
        7
    );
}

#[test]
fn pwb_v2_packet_waveform_at() {
    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .waveform_at(ChannelId::try_from(57).unwrap())
            .unwrap(),
        [513, 1027, 1541, 2055, 2569]
    );

    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .waveform_at(ChannelId::try_from(65).unwrap())
            .unwrap(),
        [3083, 3597, 4111, 4625, 5139]
    );

    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .waveform_at(ChannelId::try_from(73).unwrap())
            .unwrap(),
        [5653, 6167, 6681, 7195, 7709]
    );

    assert_eq!(
        PwbV2Packet::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .waveform_at(ChannelId::try_from(1).unwrap()),
        None
    );
}

const CHUNK_ZERO: [u8; 64] = [
    236, 40, 255, 135, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 40, 0, 118, 99, 211, 179, 2, 68, 0, 0, 236,
    40, 255, 135, 84, 2, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 0, 0, 0, 14, 90, 136, 84,
];
const CHUNK_ONE: [u8; 64] = [
    236, 40, 255, 135, 1, 0, 0, 0, 1, 0, 3, 0, 1, 0, 40, 0, 217, 96, 219, 22, 0, 0, 0, 0, 4, 0, 0,
    0, 5, 0, 6, 7, 57, 0, 5, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0, 65, 0, 5, 0, 11, 12, 13, 14,
    15, 16, 17, 18, 204, 252, 9, 110,
];
const CHUNK_TWO: [u8; 48] = [
    236, 40, 255, 135, 2, 0, 0, 0, 2, 0, 3, 1, 2, 0, 24, 0, 246, 111, 112, 132, 19, 20, 0, 0, 73,
    0, 5, 0, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 0, 0, 204, 204, 204, 204, 112, 195, 108, 175,
];
const CHUNK_ALONE: [u8; 128] = [
    236, 40, 255, 135, 2, 0, 0, 0, 2, 0, 3, 1, 0, 0, 104, 0, 240, 152, 78, 132, 2, 68, 0, 0, 236,
    40, 255, 135, 84, 2, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 5, 0, 6, 7, 57, 0, 5, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    0, 0, 65, 0, 5, 0, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 0, 0, 73, 0, 5, 0, 21, 22, 23, 24,
    25, 26, 27, 28, 29, 30, 0, 0, 204, 204, 204, 204, 1, 110, 54, 184,
];

#[test]
fn pwb_v2_packet_from_chunks_ok() {
    let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
    let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();
    let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

    let chunks = vec![chunk_zero.clone(), chunk_one.clone(), chunk_two.clone()];
    assert!(PwbV2Packet::try_from(chunks).is_ok());

    let chunks = vec![chunk_zero.clone(), chunk_two.clone(), chunk_one.clone()];
    assert!(PwbV2Packet::try_from(chunks).is_ok());

    let chunks = vec![chunk_one.clone(), chunk_zero.clone(), chunk_two.clone()];
    assert!(PwbV2Packet::try_from(chunks).is_ok());

    let chunks = vec![chunk_one.clone(), chunk_two.clone(), chunk_zero.clone()];
    assert!(PwbV2Packet::try_from(chunks).is_ok());

    let chunks = vec![chunk_two.clone(), chunk_zero.clone(), chunk_one.clone()];
    assert!(PwbV2Packet::try_from(chunks).is_ok());

    let chunks = vec![chunk_two.clone(), chunk_one.clone(), chunk_zero.clone()];
    assert!(PwbV2Packet::try_from(chunks).is_ok());

    let chunk = Chunk::try_from(&CHUNK_ALONE[..]).unwrap();
    let chunks = vec![chunk];
    assert!(PwbV2Packet::try_from(chunks).is_ok());
}

#[test]
fn pwb_v2_packet_from_chunks_device_id_mismatch() {
    let mut chunk_zero = CHUNK_ZERO;
    for triplet in PADWINGBOARDS {
        if BoardId::try_from(triplet.2).unwrap() == BoardId::try_from("00").unwrap() {
            continue;
        }
        chunk_zero[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
        let crc = !crc32c::crc32c(&chunk_zero[0..16]);
        chunk_zero[16..20].copy_from_slice(&crc.to_le_bytes()[..]);

        let chunk_zero = Chunk::try_from(&chunk_zero[..]).unwrap();
        let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();
        let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

        match PwbV2Packet::try_from(vec![chunk_zero, chunk_one, chunk_two]) {
            Err(TryPwbPacketFromChunksError::DeviceIdMismatch { found, expected }) => {
                assert_eq!(found, BoardId::try_from("00").unwrap());
                assert_eq!(expected, BoardId::try_from(triplet.2).unwrap());
            }
            _ => unreachable!(),
        }
    }

    let mut chunk_one = CHUNK_ONE;
    for triplet in PADWINGBOARDS {
        if BoardId::try_from(triplet.2).unwrap() == BoardId::try_from("00").unwrap() {
            continue;
        }
        chunk_one[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
        let crc = !crc32c::crc32c(&chunk_one[0..16]);
        chunk_one[16..20].copy_from_slice(&crc.to_le_bytes()[..]);

        let chunk_one = Chunk::try_from(&chunk_one[..]).unwrap();
        let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
        let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

        match PwbV2Packet::try_from(vec![chunk_zero, chunk_one, chunk_two]) {
            Err(TryPwbPacketFromChunksError::DeviceIdMismatch { found, expected }) => {
                assert_eq!(expected, BoardId::try_from("00").unwrap());
                assert_eq!(found, BoardId::try_from(triplet.2).unwrap());
            }
            _ => unreachable!(),
        }
    }

    let mut chunk_two = CHUNK_TWO;
    for triplet in PADWINGBOARDS {
        if BoardId::try_from(triplet.2).unwrap() == BoardId::try_from("00").unwrap() {
            continue;
        }
        chunk_two[0..4].copy_from_slice(&triplet.2.to_le_bytes()[..]);
        let crc = !crc32c::crc32c(&chunk_two[0..16]);
        chunk_two[16..20].copy_from_slice(&crc.to_le_bytes()[..]);

        let chunk_two = Chunk::try_from(&chunk_two[..]).unwrap();
        let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
        let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();

        match PwbV2Packet::try_from(vec![chunk_zero, chunk_one, chunk_two]) {
            Err(TryPwbPacketFromChunksError::DeviceIdMismatch { found, expected }) => {
                assert_eq!(expected, BoardId::try_from("00").unwrap());
                assert_eq!(found, BoardId::try_from(triplet.2).unwrap());
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn pwb_v2_packet_from_chunks_channel_id_mismatch() {
    let mut chunk_zero = CHUNK_ZERO;
    for channel in 0..=2 {
        chunk_zero[10] = channel;
        let crc = !crc32c::crc32c(&chunk_zero[0..16]);
        chunk_zero[16..20].copy_from_slice(&crc.to_le_bytes()[..]);

        let chunk_zero = Chunk::try_from(&chunk_zero[..]).unwrap();
        let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();
        let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

        match PwbV2Packet::try_from(vec![chunk_zero, chunk_one, chunk_two]) {
            Err(TryPwbPacketFromChunksError::ChannelIdMismatch { found, expected }) => {
                assert_eq!(found, AfterId::try_from('D').unwrap());
                assert_eq!(expected, AfterId::try_from(channel).unwrap());
            }
            _ => unreachable!(),
        }
    }

    let mut chunk_one = CHUNK_ONE;
    for channel in 0..=2 {
        chunk_one[10] = channel;
        let crc = !crc32c::crc32c(&chunk_one[0..16]);
        chunk_one[16..20].copy_from_slice(&crc.to_le_bytes()[..]);

        let chunk_one = Chunk::try_from(&chunk_one[..]).unwrap();
        let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
        let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

        match PwbV2Packet::try_from(vec![chunk_zero, chunk_one, chunk_two]) {
            Err(TryPwbPacketFromChunksError::ChannelIdMismatch { found, expected }) => {
                assert_eq!(expected, AfterId::try_from('D').unwrap());
                assert_eq!(found, AfterId::try_from(channel).unwrap());
            }
            _ => unreachable!(),
        }
    }

    let mut chunk_two = CHUNK_TWO;
    for channel in 0..=2 {
        chunk_two[10] = channel;
        let crc = !crc32c::crc32c(&chunk_two[0..16]);
        chunk_two[16..20].copy_from_slice(&crc.to_le_bytes()[..]);

        let chunk_two = Chunk::try_from(&chunk_two[..]).unwrap();
        let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
        let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();

        match PwbV2Packet::try_from(vec![chunk_zero, chunk_one, chunk_two]) {
            Err(TryPwbPacketFromChunksError::ChannelIdMismatch { found, expected }) => {
                assert_eq!(expected, AfterId::try_from('D').unwrap());
                assert_eq!(found, AfterId::try_from(channel).unwrap());
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn pwb_v2_packet_from_chunks_missing_chunk() {
    match PwbV2Packet::try_from(Vec::new()) {
        Err(TryPwbPacketFromChunksError::MissingChunk { position }) => {
            assert_eq!(position, 0);
        }
        _ => unreachable!(),
    }
    let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
    let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();
    let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

    let chunks = vec![chunk_one.clone(), chunk_two.clone()];
    match PwbV2Packet::try_from(chunks) {
        Err(TryPwbPacketFromChunksError::MissingChunk { position }) => {
            assert_eq!(position, 0);
        }
        _ => unreachable!(),
    }

    let chunks = vec![chunk_two.clone(), chunk_one.clone()];
    match PwbV2Packet::try_from(chunks) {
        Err(TryPwbPacketFromChunksError::MissingChunk { position }) => {
            assert_eq!(position, 0);
        }
        _ => unreachable!(),
    }

    let chunks = vec![chunk_zero.clone(), chunk_two.clone()];
    match PwbV2Packet::try_from(chunks) {
        Err(TryPwbPacketFromChunksError::MissingChunk { position }) => {
            assert_eq!(position, 1);
        }
        _ => unreachable!(),
    }

    let chunks = vec![chunk_two.clone(), chunk_zero.clone()];
    match PwbV2Packet::try_from(chunks) {
        Err(TryPwbPacketFromChunksError::MissingChunk { position }) => {
            assert_eq!(position, 1);
        }
        _ => unreachable!(),
    }
}

#[test]
fn pwb_v2_packet_from_chunks_missing_end_of_message_chunk() {
    let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
    let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();

    let chunks = vec![chunk_zero.clone(), chunk_one.clone()];
    assert!(matches!(
        PwbV2Packet::try_from(chunks),
        Err(TryPwbPacketFromChunksError::MissingEndOfMessageChunk)
    ));

    let chunks = vec![chunk_zero.clone()];
    assert!(matches!(
        PwbV2Packet::try_from(chunks),
        Err(TryPwbPacketFromChunksError::MissingEndOfMessageChunk)
    ));
}

#[test]
fn pwb_v2_packet_from_chunks_misplaced_end_of_message_chunk() {
    let mut chunk_zero = CHUNK_ZERO;
    chunk_zero[11] = 1;
    let crc = !crc32c::crc32c(&chunk_zero[0..16]);
    chunk_zero[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
    let chunk_zero = Chunk::try_from(&chunk_zero[..]).unwrap();
    let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();
    let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

    let chunks = vec![chunk_zero.clone(), chunk_one.clone(), chunk_two.clone()];
    match PwbV2Packet::try_from(chunks) {
        Err(TryPwbPacketFromChunksError::MisplacedEndOfMessageChunk { position }) => {
            assert_eq!(position, 0);
        }
        _ => unreachable!(),
    }

    let mut chunk_one = CHUNK_ONE;
    chunk_one[11] = 1;
    let crc = !crc32c::crc32c(&chunk_one[0..16]);
    chunk_one[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
    let chunk_one = Chunk::try_from(&chunk_one[..]).unwrap();
    let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
    let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

    let chunks = vec![chunk_zero.clone(), chunk_one.clone(), chunk_two.clone()];
    match PwbV2Packet::try_from(chunks) {
        Err(TryPwbPacketFromChunksError::MisplacedEndOfMessageChunk { position }) => {
            assert_eq!(position, 1);
        }
        _ => unreachable!(),
    }
}

#[test]
fn pwb_v2_packet_from_chunks_payload_length_mismatch() {
    let mut chunk_zero = CHUNK_ZERO;
    chunk_zero[11] = 1;
    chunk_zero[12] = 2;
    let crc = !crc32c::crc32c(&chunk_zero[0..16]);
    chunk_zero[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
    let chunk_zero = Chunk::try_from(&chunk_zero[..]).unwrap();

    let mut chunk_two = CHUNK_TWO;
    chunk_two[11] = 0;
    chunk_two[12] = 0;
    let crc = !crc32c::crc32c(&chunk_two[0..16]);
    chunk_two[16..20].copy_from_slice(&crc.to_le_bytes()[..]);
    let chunk_two = Chunk::try_from(&chunk_two[..]).unwrap();

    let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();
    let chunks = vec![chunk_zero, chunk_one, chunk_two];
    match PwbV2Packet::try_from(chunks) {
        Err(TryPwbPacketFromChunksError::PayloadLengthMismatch { found, expected }) => {
            assert_eq!(found, 40);
            assert_eq!(expected, 24);
        }
        _ => unreachable!(),
    }
}

#[test]
fn pwb_v2_packet_from_chunks_bad_payload() {
    let mut chunk_zero = CHUNK_ZERO;
    chunk_zero[20] = 1;
    let crc = !crc32c::crc32c(&chunk_zero[20..60]);
    chunk_zero[60..].copy_from_slice(&crc.to_le_bytes()[..]);
    let chunk_zero = Chunk::try_from(&chunk_zero[..]).unwrap();

    let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();
    let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();
    let chunks = vec![chunk_zero, chunk_one, chunk_two];
    assert!(matches!(
        PwbV2Packet::try_from(chunks),
        Err(TryPwbPacketFromChunksError::BadPayload(_))
    ));
}

#[test]
fn pwb_packet_good() {
    let chunk_zero = Chunk::try_from(&CHUNK_ZERO[..]).unwrap();
    let chunk_one = Chunk::try_from(&CHUNK_ONE[..]).unwrap();
    let chunk_two = Chunk::try_from(&CHUNK_TWO[..]).unwrap();

    let chunks = vec![chunk_zero.clone(), chunk_one.clone(), chunk_two.clone()];
    assert!(PwbPacket::try_from(chunks).is_ok());

    assert!(PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).is_ok());
}

#[test]
fn pwb_packet_to_string() {
    let packet = PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap();
    assert_eq!(
        format!("{packet}"),
        "Format revision: 2
AFTER ID: D
Compression: Raw
Trigger source: External
MAC address: [236, 40, 255, 135, 84, 2]
Trigger delay: 1
Trigger timestamp: 2
Last SCA cell: 3
Requested samples: 5
Channels sent: [57, 65, 73]
Channels over threshold: [1, 9, 17]
Event counter: 4
FIFO max depth: 5
Event descriptor write depth: 6
Event descriptor read depth: 7"
    );
}

#[test]
fn pwb_packet_version() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .packet_version(),
        2
    );
}

#[test]
fn pwb_packet_after_id() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .after_id(),
        AfterId::try_from('D').unwrap()
    );
}

#[test]
fn pwb_packet_compression() {
    assert!(matches!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .compression(),
        Compression::Raw
    ));
}

#[test]
fn pwb_packet_trigger_source() {
    assert!(matches!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .trigger_source(),
        Trigger::External
    ));
}

#[test]
fn pwb_packet_board_id() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .board_id(),
        BoardId::try_from("00").unwrap()
    );
}

#[test]
fn pwb_packet_trigger_delay() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .trigger_delay(),
        1
    );
}

#[test]
fn pwb_packet_trigger_timestamp() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .trigger_timestamp(),
        2
    );
}

#[test]
fn pwb_packet_last_sca_cell() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .last_sca_cell(),
        3
    );
}

#[test]
fn pwb_packet_requested_samples() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .requested_samples(),
        5
    );
}

#[test]
fn pwb_packet_channels_sent() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .channels_sent(),
        [
            ChannelId::try_from(57).unwrap(),
            ChannelId::try_from(65).unwrap(),
            ChannelId::try_from(73).unwrap()
        ]
    );
}

#[test]
fn pwb_packet_channels_over_threshold() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .channels_over_threshold(),
        [
            ChannelId::try_from(1).unwrap(),
            ChannelId::try_from(9).unwrap(),
            ChannelId::try_from(17).unwrap()
        ]
    );
}

#[test]
fn pwb_packet_event_counter() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .event_counter()
            .unwrap(),
        4
    );
}

#[test]
fn pwb_packet_fifo_max_depth() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .fifo_max_depth()
            .unwrap(),
        5
    );
}

#[test]
fn pwb_packet_event_descriptor_write_depth() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .event_descriptor_write_depth()
            .unwrap(),
        6
    );
}

#[test]
fn pwb_packet_event_descriptor_read_depth() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .event_descriptor_read_depth()
            .unwrap(),
        7
    );
}

#[test]
fn pwb_packet_waveform_at() {
    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .waveform_at(ChannelId::try_from(57).unwrap())
            .unwrap(),
        [513, 1027, 1541, 2055, 2569]
    );

    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .waveform_at(ChannelId::try_from(65).unwrap())
            .unwrap(),
        [3083, 3597, 4111, 4625, 5139]
    );

    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .waveform_at(ChannelId::try_from(73).unwrap())
            .unwrap(),
        [5653, 6167, 6681, 7195, 7709]
    );

    assert_eq!(
        PwbPacket::try_from(&ODD_PWB_V2_PACKET[..])
            .unwrap()
            .waveform_at(ChannelId::try_from(1).unwrap()),
        None
    );
}

#[test]
fn suppression_baseline_short_slice() {
    let slice = [0; 67];
    match suppression_baseline(0, &slice) {
        Err(CalculateSuppressionBaselineError { found }) => {
            assert_eq!(found, 67);
        }
        _ => unreachable!(),
    }
}

#[test]
fn suppression_baseline_ok() {
    let mut slice = vec![0; 68];
    match suppression_baseline(0, &slice) {
        Ok(Some(value)) => {
            assert_eq!(value, 0);
        }
        _ => unreachable!(),
    }

    slice[0] = i16::MAX;
    slice[1] = i16::MAX;
    slice[2] = i16::MAX;
    slice[3] = i16::MAX;
    match suppression_baseline(0, &slice) {
        Ok(Some(value)) => {
            assert_eq!(value, 0);
        }
        _ => unreachable!(),
    }

    slice[67] = 64;
    match suppression_baseline(0, &slice) {
        Ok(Some(value)) => {
            assert_eq!(value, 1);
        }
        _ => unreachable!(),
    }
}
