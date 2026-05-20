use super::super::geometry::Leaf;

pub(super) fn build(x: f32, y: f32, angle: f32, size: f32) -> Leaf {
    let cos_a = angle.to_radians().cos();
    let sin_a = angle.to_radians().sin();

    // Transform a normalized point to screen coordinates
    let t = |nx: f32, ny: f32| -> (f32, f32) {
        (x + (-ny * cos_a + nx * sin_a) * size,
         y + ( ny * sin_a + nx * cos_a) * size)
    };

    let (px, py) = t(0.0, 0.0);           // petiole start at origin
    let (bx, by) = t(0.0, -0.3125);       // fan base
    let (blx, bly) = t(-0.25, -0.5625);
    let (lcx, lcy) = t(-0.875, -0.5625);
    let (lax, lay) = t(-1.0, -0.6875);
    let (dlx, dly) = t(0.0, -1.1875);
    let (dl_c1x, dl_c1y) = t(-0.875, -1.3125);
    let (dl_c2x, dl_c2y) = t(0.0, -1.5625);
    let (drx, dry) = t(1.0, -0.6875);
    let (dr_c1x, dr_c1y) = t(0.0, -1.5625);
    let (dr_c2x, dr_c2y) = t(0.875, -1.3125);
    let (rax, ray) = t(0.875, -0.5625);
    let (rcx, rcy) = t(0.25, -0.5625);
    let (srx, sry) = t(0.0, -0.3125);

    Leaf {
        px, py,
        bx, by,
        blx, bly, bl_radius: 0.25 * size,
        lcx, lcy,
        lax, lay, la_radius: 0.125 * size,
        dlx, dly, dl_c1x, dl_c1y, dl_c2x, dl_c2y,
        drx, dry, dr_c1x, dr_c1y, dr_c2x, dr_c2y,
        rax, ray, ra_radius: 0.125 * size,
        rcx, rcy,
        srx, sry, sr_radius: 0.25 * size,
        stem_sweep: false,
        corner_sweep: true,
    }
}