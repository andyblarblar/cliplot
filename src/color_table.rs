//! Algorithm to find a different color for each channel, deterministically.

use plotters::style::RGBColor;

// Algorithm from https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/

/// Gets the appropriate color for each channel
pub fn get_color_for_channels(channel_count: usize) -> Vec<RGBColor> {
    const GOLDEN_RATIO: f64 = 0.618033988749895;

    let mut out = Vec::new();

    // dont use rand here to make deterministic
    let mut h = 0.3;
    for _ in 0..channel_count {
        h += GOLDEN_RATIO;
        h %= 1.0;
        out.push(hsv_to_rgb(h, 0.9, 0.95));
    }

    out
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> RGBColor {
    let h_i = (h * 6.0) as u64;
    let f = h * 6.0 - h_i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match h_i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => {
            unreachable!()
        }
    };

    RGBColor((r * 256.0) as u8, (g * 256.0) as u8, (b * 256.0) as u8)
}
