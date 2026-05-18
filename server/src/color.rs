use rand::Rng;

use crate::protocol::Rgb;

pub fn random_pair<R: Rng + ?Sized>(rng: &mut R) -> (Rgb, Rgb) {
    let primary_hue: f32 = rng.gen_range(0.0..360.0);
    let offset: f32 = rng.gen_range(60.0..300.0);
    let outline_hue = (primary_hue + offset).rem_euclid(360.0);
    let primary = hsl_to_rgb(primary_hue, 0.75, 0.55);
    let outline = hsl_to_rgb(outline_hue, 0.85, 0.65);
    (primary, outline)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Rgb {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    [
        ((r1 + m).clamp(0.0, 1.0) * 255.0).round() as u8,
        ((g1 + m).clamp(0.0, 1.0) * 255.0).round() as u8,
        ((b1 + m).clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}
