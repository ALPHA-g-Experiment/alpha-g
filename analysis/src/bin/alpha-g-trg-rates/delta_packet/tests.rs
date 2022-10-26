use super::*;

const TRG_V3_PACKET: [u8; 80] = [
    255, 0, 0, 0, 0, 0, 0, 128, 254, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 6, 0,
    0, 0, 7, 0, 0, 0, 8, 0, 0, 128, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 10, 0, 11, 0, 0, 0,
    0, 0, 0, 0, 12, 0, 0, 0, 13, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 224,
];

#[test]
fn delta_packet_ok() {
    let mut current = TRG_V3_PACKET;
    current[8] = 255;

    current[12] = 100;
    current[16] = 200;
    current[20] = 56;
    current[40] = 135;
    current[44] = 119;

    current[4] = 100;
    current[76] = 100;

    let current_packet = TrgPacket::try_from(&current[..]).unwrap();
    let previous = TrgPacket::try_from(&TRG_V3_PACKET[..]).unwrap();
    let delta_packet = DeltaPacket::try_from(&current_packet, &previous).unwrap();

    assert_eq!(delta_packet.timestamp, 1);
    assert_eq!(delta_packet.output_counter, 100);
    assert_eq!(delta_packet.input_counter, 197);
    assert_eq!(delta_packet.pulser_counter, 52);
    assert_eq!(delta_packet.drift_veto_counter, 133);
    assert_eq!(delta_packet.scaledown_counter, 118);

    current[8] = 253;
    let current = TrgPacket::try_from(&current[..]).unwrap();
    let delta_packet = DeltaPacket::try_from(&current, &previous).unwrap();
    assert_eq!(delta_packet.timestamp, u32::MAX);
}

#[test]
fn delta_packet_corrupted_output_counter() {
    let current = TrgPacket::try_from(&TRG_V3_PACKET[..]).unwrap();

    let mut previous = TRG_V3_PACKET;
    previous[4] = 1;
    previous[12] = 1;
    previous[76] = 1;
    let previous = TrgPacket::try_from(&previous[..]).unwrap();

    assert_eq!(
        DeltaPacket::try_from(&current, &previous).unwrap_err(),
        String::from("corrupted output counter")
    );
}

#[test]
fn delta_packet_corrupted_input_counter() {
    let current = TrgPacket::try_from(&TRG_V3_PACKET[..]).unwrap();

    let mut previous = TRG_V3_PACKET;
    previous[16] = 4;
    let previous = TrgPacket::try_from(&previous[..]).unwrap();

    assert_eq!(
        DeltaPacket::try_from(&current, &previous).unwrap_err(),
        String::from("corrupted input counter")
    );
}

#[test]
fn delta_packet_non_incrementing_counter() {
    let packet = TrgPacket::try_from(&TRG_V3_PACKET[..]).unwrap();
    assert_eq!(
        DeltaPacket::try_from(&packet, &packet).unwrap_err(),
        String::from("non-incrementing counter")
    );
}

#[test]
fn delta_packet_corrupted_pulser_counter() {
    let current = TrgPacket::try_from(&TRG_V3_PACKET[..]).unwrap();

    let mut previous = TRG_V3_PACKET;
    previous[20] = 5;
    let previous = TrgPacket::try_from(&previous[..]).unwrap();

    assert_eq!(
        DeltaPacket::try_from(&current, &previous).unwrap_err(),
        String::from("corrupted pulser counter")
    );
}

#[test]
fn delta_packet_corrupted_drift_veto_counter() {
    let current = TrgPacket::try_from(&TRG_V3_PACKET[..]).unwrap();

    let mut previous = TRG_V3_PACKET;
    previous[40] = 3;
    let previous = TrgPacket::try_from(&previous[..]).unwrap();

    assert_eq!(
        DeltaPacket::try_from(&current, &previous).unwrap_err(),
        String::from("corrupted drift veto counter")
    );
}

#[test]
fn delta_packet_corrupted_scaledown_counter() {
    let current = TrgPacket::try_from(&TRG_V3_PACKET[..]).unwrap();

    let mut previous = TRG_V3_PACKET;
    previous[44] = 2;
    let previous = TrgPacket::try_from(&previous[..]).unwrap();

    assert_eq!(
        DeltaPacket::try_from(&current, &previous).unwrap_err(),
        String::from("corrupted scaledown counter")
    );
}
