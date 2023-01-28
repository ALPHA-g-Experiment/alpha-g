use super::*;

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

#[test]
fn packet_correctness() {
    let packet = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };

    assert!(matches!(packet.correctness(), Correctness::Good));

    let packet = Packet {
        adc_packet: vec![0],
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };

    assert!(matches!(packet.correctness(), Correctness::Bad));
}

#[test]
fn packet_detector() {
    let packet = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };

    assert!(matches!(packet.detector(), Detector::Bv));

    let packet = Packet {
        adc_packet: vec![0],
        bank_name: String::from("C09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };

    assert!(matches!(packet.detector(), Detector::Tpc));
}

#[test]
fn packet_keep_bit() {
    let packet = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };

    assert!(packet.keep_bit().unwrap());

    let packet = Packet {
        adc_packet: vec![0],
        bank_name: String::from("C09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };

    assert!(packet.keep_bit().is_none());
}

#[test]
fn packet_overflow() {
    let packet = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(matches!(packet.overflow(), Some(Overflow::Neither)));

    let mut adc_packet = LONG_ADC_V3_PACKET.to_vec();
    adc_packet[160] = 128;
    adc_packet[161] = 0;
    let packet = Packet {
        adc_packet,
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(matches!(packet.overflow(), Some(Overflow::Negative)));

    let mut adc_packet = LONG_ADC_V3_PACKET.to_vec();
    adc_packet[160] = 127;
    adc_packet[161] = 252;
    let packet = Packet {
        adc_packet,
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(matches!(packet.overflow(), Some(Overflow::Positive)));

    let mut adc_packet = LONG_ADC_V3_PACKET.to_vec();
    adc_packet[160] = 127;
    adc_packet[161] = 252;
    adc_packet.insert(160, 128);
    adc_packet.insert(161, 0);
    let packet = Packet {
        adc_packet,
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(matches!(packet.overflow(), Some(Overflow::Both)));

    let packet = Packet {
        adc_packet: vec![0],
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet.overflow().is_none());

    let packet = Packet {
        adc_packet: SHORT_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet.overflow().is_none());
}

#[test]
fn packet_passes_correctness_filter() {
    let filter_one = Filter {
        correctness: Some(Correctness::Good),
        ..Filter::default()
    };
    let packet_one = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_one.passes_filter(&filter_one));

    let filter_two = Filter {
        correctness: Some(Correctness::Bad),
        ..Filter::default()
    };
    let packet_two = Packet {
        adc_packet: vec![0],
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_two.passes_filter(&filter_two));

    assert!(!packet_one.passes_filter(&filter_two));
    assert!(!packet_two.passes_filter(&filter_one));
}

#[test]
fn packet_passes_detector_filter() {
    let filter_one = Filter {
        detector: Some(Detector::Tpc),
        ..Filter::default()
    };
    let packet_one = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("C09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_one.passes_filter(&filter_one));

    let filter_two = Filter {
        detector: Some(Detector::Bv),
        ..Filter::default()
    };
    let packet_two = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_two.passes_filter(&filter_two));

    assert!(!packet_one.passes_filter(&filter_two));
    assert!(!packet_two.passes_filter(&filter_one));
}

#[test]
fn packet_passes_keep_bit_filter() {
    let filter_one = Filter {
        keep_bit: Some(true),
        ..Filter::default()
    };
    let packet_one = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("C09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_one.passes_filter(&filter_one));

    let filter_two = Filter {
        keep_bit: Some(false),
        ..Filter::default()
    };
    let packet_two = Packet {
        adc_packet: SHORT_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("C09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_two.passes_filter(&filter_two));

    assert!(!packet_one.passes_filter(&filter_two));
    assert!(!packet_two.passes_filter(&filter_one));
}

#[test]
fn packet_passes_overflow_filter() {
    let filter_one = Filter {
        overflow: Some(Overflow::Neither),
        ..Filter::default()
    };
    let packet_one = Packet {
        adc_packet: LONG_ADC_V3_PACKET.to_vec(),
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_one.passes_filter(&filter_one));

    let filter_two = Filter {
        overflow: Some(Overflow::Negative),
        ..Filter::default()
    };
    let mut adc_packet = LONG_ADC_V3_PACKET.to_vec();
    adc_packet[160] = 128;
    adc_packet[161] = 0;
    let packet_two = Packet {
        adc_packet,
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_two.passes_filter(&filter_two));

    let filter_three = Filter {
        overflow: Some(Overflow::Positive),
        ..Filter::default()
    };
    let mut adc_packet = LONG_ADC_V3_PACKET.to_vec();
    adc_packet[160] = 127;
    adc_packet[161] = 252;
    let packet_three = Packet {
        adc_packet,
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_three.passes_filter(&filter_three));

    let filter_four = Filter {
        overflow: Some(Overflow::Both),
        ..Filter::default()
    };
    let mut adc_packet = LONG_ADC_V3_PACKET.to_vec();
    adc_packet[160] = 127;
    adc_packet[161] = 252;
    adc_packet.insert(160, 128);
    adc_packet.insert(161, 0);
    let packet_four = Packet {
        adc_packet,
        bank_name: String::from("B09A"),
        a16_suppression: Some(500.0),
        a32_suppression: Some(1500.0),
    };
    assert!(packet_four.passes_filter(&filter_four));

    assert!(!packet_one.passes_filter(&filter_two));
    assert!(!packet_one.passes_filter(&filter_three));
    assert!(!packet_one.passes_filter(&filter_four));
    assert!(!packet_two.passes_filter(&filter_one));
    assert!(!packet_two.passes_filter(&filter_three));
    assert!(!packet_two.passes_filter(&filter_four));
    assert!(!packet_three.passes_filter(&filter_one));
    assert!(!packet_three.passes_filter(&filter_two));
    assert!(!packet_three.passes_filter(&filter_four));
    assert!(!packet_four.passes_filter(&filter_one));
    assert!(!packet_four.passes_filter(&filter_two));
    assert!(!packet_four.passes_filter(&filter_three));
}
