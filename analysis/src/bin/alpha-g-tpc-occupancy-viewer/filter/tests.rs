use super::*;
use alpha_g_detector::alpha16::AdcPacket;
use alpha_g_detector::padwing::PwbPacket;

const SHORT_ADC_V3_PACKET: [u8; 16] = [1, 3, 0, 1, 2, 3, 2, 187, 0, 0, 0, 4, 224, 0, 0, 0];
const LONG_ADC_V3_PACKET: [u8; 166] = [
    1, 3, 0, 1, 2, 3, 2, 187, 0, 0, 0, 4, 0, 0, 216, 128, 57, 104, 142, 82, 0, 0, 0, 0, 0, 0, 0, 5,
    0, 0, 0, 6, 255, 224, 255, 225, 255, 226, 255, 227, 255, 228, 255, 229, 255, 230, 255, 231,
    255, 232, 255, 233, 255, 234, 255, 235, 255, 236, 255, 237, 255, 238, 255, 239, 255, 240, 255,
    241, 255, 242, 255, 243, 255, 244, 255, 245, 255, 246, 255, 247, 255, 248, 255, 249, 255, 250,
    255, 251, 255, 252, 255, 253, 255, 254, 255, 255, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0,
    8, 0, 9, 0, 10, 0, 11, 0, 12, 0, 13, 0, 14, 0, 15, 0, 16, 0, 17, 0, 18, 0, 19, 0, 20, 0, 21, 0,
    22, 0, 23, 0, 24, 0, 25, 0, 26, 0, 27, 0, 28, 0, 29, 0, 30, 0, 31, 0, 32, 0, 0, 240, 34, 0, 0,
];

const ODD_PWB_V2_PACKET: [u8; 120] = [
    2, 68, 0, 0, 236, 40, 255, 135, 84, 2, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 5, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 5, 0, 6, 7, 49, 0, 5, 0, 255, 7, 3, 0,
    5, 0, 7, 0, 9, 0, 0, 0, 57, 0, 5, 0, 0, 248, 13, 0, 15, 0, 17, 0, 19, 0, 0, 0, 65, 0, 5, 0,
    255, 7, 0, 248, 25, 0, 27, 0, 29, 0, 0, 0, 73, 0, 5, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0,
    204, 204, 204, 204,
];

#[test]
fn packet_num_anode_wires() {
    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&SHORT_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert_eq!(packet.num_anode_wires(), 0);

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&LONG_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert_eq!(packet.num_anode_wires(), 1);
}

#[test]
fn packet_num_pads() {
    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&SHORT_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert_eq!(packet.num_pads(), 4);

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&LONG_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert_eq!(packet.num_pads(), 4);
}

#[test]
fn packet_passes_anode_wires_filter() {
    let filter = Filter {
        min_anode_wires: Some(1),
        ..Default::default()
    };

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&SHORT_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(!packet.passes_filter(filter));

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&LONG_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(packet.passes_filter(filter));

    let filter = Filter {
        max_anode_wires: Some(0),
        ..Default::default()
    };

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&SHORT_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(packet.passes_filter(filter));

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&LONG_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(!packet.passes_filter(filter));
}

#[test]
fn packet_passes_pads_filter() {
    let filter = Filter {
        min_pads: Some(5),
        ..Default::default()
    };

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&SHORT_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(!packet.passes_filter(filter));

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&LONG_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(!packet.passes_filter(filter));

    let filter = Filter {
        min_pads: Some(4),
        ..Default::default()
    };

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&SHORT_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(packet.passes_filter(filter));

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&LONG_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(packet.passes_filter(filter));

    let filter = Filter {
        max_pads: Some(3),
        ..Default::default()
    };

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&SHORT_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(!packet.passes_filter(filter));

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&LONG_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(!packet.passes_filter(filter));

    let filter = Filter {
        max_pads: Some(4),
        ..Default::default()
    };

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&SHORT_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(packet.passes_filter(filter));

    let packet = Packet {
        pwb_packets: vec![PwbPacket::try_from(&ODD_PWB_V2_PACKET[..]).unwrap()],
        adc_packets: vec![AdcPacket::try_from(&LONG_ADC_V3_PACKET[..]).unwrap()],
        serial_number: 0,
        run_number: 0,
    };
    assert!(packet.passes_filter(filter));
}
