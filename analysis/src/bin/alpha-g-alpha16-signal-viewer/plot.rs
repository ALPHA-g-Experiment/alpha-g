use crate::Packet;
use alpha_g_detector::alpha16::ChannelId::{A16, A32};
use alpha_g_detector::alpha16::{AdcPacket, ADC16_RATE, ADC32_RATE, ADC_MAX, ADC_MIN};
use pgfplots::{
    axis::{plot::*, *},
    Picture,
};

/// Create an empty axis environment.
pub fn empty_picture() -> Picture {
    let axis = Axis::new();
    Picture::from(axis)
}

/// Create a [`Picture`] based on an input [`Packet`].
pub fn create_picture(packet: &Packet) -> Picture {
    let mut legend = Vec::new();
    let mut axis = Axis::new();
    if let Ok(adc_packet) = AdcPacket::try_from(&packet.adc_packet[..]) {
        axis.set_title(format!("{} Waveform", packet.bank_name));
        axis.set_x_label(format!(
            "Samples~[{} ns]",
            1e9 / match adc_packet.channel_id() {
                A16(_) => ADC16_RATE,
                A32(_) => ADC32_RATE,
            }
        ));
        axis.set_y_label("Amplitude~[a.u.]");
        axis.add_key(AxisKey::Custom(format!("ymin={ADC_MIN}, ymax={ADC_MAX}")));

        let last_index = if adc_packet.waveform().is_empty() {
            adc_packet.requested_samples()
        } else {
            adc_packet.waveform().len()
        };
        let suppression_threshold = match adc_packet.channel_id() {
            A16(_) => packet.a16_suppression,
            A32(_) => packet.a32_suppression,
        };
        if let Some(threshold) = suppression_threshold {
            if let Some(baseline) = adc_packet.suppression_baseline() {
                let mut suppression = Plot2D::new();
                for (mut x, mut y) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                    x *= last_index as f64;
                    y = y * 2.0 * threshold + f64::from(baseline) - threshold;
                    suppression.coordinates.push((x, y).into());
                }
                suppression.add_key(PlotKey::Custom(
                    "fill=gray!20, draw=gray!20, area legend".to_string(),
                ));
                axis.plots.push(suppression);
                legend.push(String::from("Data suppression"));
            }
        }

        if !adc_packet.waveform().is_empty() {
            let mut signal = Plot2D::new();
            signal.coordinates = adc_packet
                .waveform()
                .iter()
                .enumerate()
                .map(|c| (c.0 as f64, f64::from(*c.1)).into())
                .collect();
            axis.plots.push(signal);
            legend.push(String::from("Waveform"));
        }

        axis.add_key(AxisKey::Custom(format!(
            "legend entries={{{}}}",
            legend.join(",")
        )));
        axis.add_key(AxisKey::Custom("legend style={font=\\tiny}".to_string()));
    }

    Picture::from(axis)
}
