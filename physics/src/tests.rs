use super::*;
use uom::si::frequency::megahertz;
use uom::si::length::millimeter;

#[test]
fn anode_wires_radius() {
    let r = Length::new::<millimeter>(182.0);
    assert_eq!(ANODE_WIRES_RADIUS, r);
}

#[test]
fn trigger_clock_frequency() {
    let f = Frequency::new::<megahertz>(62.5);
    assert_eq!(TRG_CLOCK_FREQ, f);
}
