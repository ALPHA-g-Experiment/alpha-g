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
