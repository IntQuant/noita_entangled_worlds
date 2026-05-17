use image::{Pixel, Rgba};

pub fn compare_rgb(a: Rgba<u8>, b: Rgba<u8>) -> bool {
    a.channels()[0..3] == b.channels()[0..3]
}

pub fn set_rgb(a: &mut Rgba<u8>, b: Rgba<u8>) {
    for i in 0..3 {
        a.channels_mut()[i] = b.channels()[i];
    }
}

pub fn invert(mut a: Rgba<u8>) -> Rgba<u8> {
    for i in 0..3 {
        a.channels_mut()[i] = 255 - a.channels()[i];
    }
    a
}

pub fn f_to_u(n: f64) -> u8 {
    255.0_f64.min(0.0_f64.max(n.round())) as u8
}

pub fn to_u8(c: [f64; 4]) -> [u8; 4] {
    [f_to_u(c[0]), f_to_u(c[1]), f_to_u(c[2]), f_to_u(c[3])]
}

pub fn get_lch(color: [f64; 4]) -> (f64, f64, f64) {
    let c = (color[1].powi(2) + color[2].powi(2)).sqrt();
    let h = color[2].atan2(color[1]);
    (color[0], c, h)
}
pub fn rgb_to_oklch(color: &mut [f64; 4]) {
    let mut l = 0.4122214694707629 * color[0]
        + 0.5363325372617349 * color[1]
        + 0.0514459932675022 * color[2];
    let mut m = 0.2119034958178251 * color[0]
        + 0.6806995506452344 * color[1]
        + 0.1073969535369405 * color[2];
    let mut s = 0.0883024591900564 * color[0]
        + 0.2817188391361215 * color[1]
        + 0.6299787016738222 * color[2];

    l = l.cbrt();
    m = m.cbrt();
    s = s.cbrt();

    color[0] = 0.210454268309314 * l + 0.7936177747023054 * m - 0.0040720430116193 * s;
    color[1] = 1.9779985324311684 * l - 2.42859224204858 * m + 0.450593709617411 * s;
    color[2] = 0.0259040424655478 * l + 0.7827717124575296 * m - 0.8086757549230774 * s;
}

pub fn oklch_to_rgb(color: &mut [f64; 4]) {
    let mut l = color[0] + 0.3963377773761749 * color[1] + 0.2158037573099136 * color[2];
    let mut m = color[0] - 0.1055613458156586 * color[1] - 0.0638541728258133 * color[2];
    let mut s = color[0] - 0.0894841775298119 * color[1] - 1.2914855480194092 * color[2];

    l = l.powi(3);
    m = m.powi(3);
    s = s.powi(3);

    color[0] = 4.07674163607596 * l - 3.3077115392580635 * m + 0.2309699031821046 * s;
    color[1] = -1.2684379732850317 * l + 2.6097573492876887 * m - 0.3413193760026572 * s;
    color[2] = -0.0041960761386754 * l - 0.7034186179359363 * m + 1.7076146940746116 * s;
}

pub fn shift_hue_by(color: &mut [f64; 4], diff: f64) {
    let tau = std::f64::consts::TAU;
    let diff = tau * diff / 360.0;
    let (_, c, hue) = get_lch(*color);
    let mut new_hue = (hue + diff) % tau;
    if new_hue.is_sign_negative() {
        new_hue += tau;
    }
    color[1] = c * new_hue.cos();
    color[2] = c * new_hue.sin();
}

pub fn shift_hue(diff: f64, color: &mut [f64; 4]) {
    rgb_to_oklch(color);
    shift_hue_by(color, diff);
    oklch_to_rgb(color);
}

pub fn rgb_to_hex(rgb: [u8; 4]) -> String {
    format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}
