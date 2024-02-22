use alpha_g_detector::chronobox;
use uom::si::f64::*;

/// Frequency of the timestamp counter clock.
pub const TIMESTAMP_CLOCK_FREQ: Frequency = Frequency {
    dimension: uom::lib::marker::PhantomData,
    units: uom::lib::marker::PhantomData,
    value: chronobox::TIMESTAMP_CLOCK_FREQ,
};

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::frequency::megahertz;

    #[test]
    fn chronobox_timestamp_counter_frequency() {
        let f = Frequency::new::<megahertz>(10.0);
        assert_eq!(TIMESTAMP_CLOCK_FREQ, f);
    }
}
