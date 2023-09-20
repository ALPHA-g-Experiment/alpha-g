use super::*;
use uom::si::angle::radian;
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

#[test]
fn spacepoint_x_coordinate() {
    let p = SpacePoint {
        r: Length::new::<millimeter>(5.0),
        phi: Angle::new::<radian>(4.0f64.atan2(3.0)),
        z: Length::new::<millimeter>(0.0),
    };

    let diff = (p.x() - Length::new::<millimeter>(3.0)).abs();
    assert!(diff < Length::new::<millimeter>(1e-6));
}

#[test]
fn spacepoint_y_coordinate() {
    let p = SpacePoint {
        r: Length::new::<millimeter>(5.0),
        phi: Angle::new::<radian>(4.0f64.atan2(3.0)),
        z: Length::new::<millimeter>(0.0),
    };

    let diff = (p.y() - Length::new::<millimeter>(4.0)).abs();
    assert!(diff < Length::new::<millimeter>(1e-6));
}
