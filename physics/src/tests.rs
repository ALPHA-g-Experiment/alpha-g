use super::*;
use uom::si::length::millimeter;

#[test]
fn anode_wires_radius() {
    let r = Length::new::<millimeter>(182.0);
    assert_eq!(ANODE_WIRES_RADIUS, r);
}
