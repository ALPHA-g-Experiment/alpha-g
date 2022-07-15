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
