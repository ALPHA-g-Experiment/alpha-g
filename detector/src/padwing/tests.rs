use super::*;

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
    assert!(matches!(AfterId::try_from('a').unwrap(), AfterId::A));
    assert!(matches!(AfterId::try_from('b').unwrap(), AfterId::B));
    assert!(matches!(AfterId::try_from('c').unwrap(), AfterId::C));
    assert!(matches!(AfterId::try_from('d').unwrap(), AfterId::D));

    for i in 'e'..'z' {
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
