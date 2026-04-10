use slint::*;
use crate::{ app, render, tree };

impl From<&render::geometry::Segment> for app::SegmentData {
    fn from(s: &render::geometry::Segment) -> Self {
        app::SegmentData {
            x1: s.x1, y1: s.y1,
            x2: s.x2, y2: s.y2,
            x3: s.x3, y3: s.y3,
            x4: s.x4, y4: s.y4,
            radius: s.radius,
            sweep: s.sweep,
        }
    }
}

impl From<&render::geometry::Leaf> for app::LeafData {
    fn from(l: &render::geometry::Leaf) -> Self {
        app::LeafData {
            px: l.px, py: l.py,
            bx: l.bx, by: l.by,
            blx: l.blx, bly: l.bly, bl_radius: l.bl_radius,
            lcx: l.lcx, lcy: l.lcy,
            lax: l.lax, lay: l.lay, la_radius: l.la_radius,
            dlx: l.dlx, dly: l.dly, dl_c1x: l.dl_c1x, dl_c1y: l.dl_c1y, dl_c2x: l.dl_c2x, dl_c2y: l.dl_c2y,
            drx: l.drx, dry: l.dry, dr_c1x: l.dr_c1x, dr_c1y: l.dr_c1y, dr_c2x: l.dr_c2x, dr_c2y: l.dr_c2y,
            rax: l.rax, ray: l.ray, ra_radius: l.ra_radius,
            rcx: l.rcx, rcy: l.rcy,
            srx: l.srx, sry: l.sry, sr_radius: l.sr_radius,
            stem_sweep: l.stem_sweep,
            corner_sweep: l.corner_sweep,
        }
    }
}

pub(crate) fn tree_to_model(tree: &tree::node::Tree, tree_config: &tree::node::TreeConfig, render_config: &render::RenderConfig, origin: (f32, f32)) -> (ModelRc<app::SegmentData>, ModelRc<app::LeafData>) {
    let mut segments = Vec::new();
    let mut leaves = Vec::new();
    render::geometry::nodes_to_renderables(tree, 0, tree_config, render_config, origin, &mut segments, &mut leaves);
    let slint_segments: Vec<app::SegmentData> = segments.iter().map(app::SegmentData::from).collect();
    let slint_leaves: Vec<app::LeafData> = leaves.iter().map(app::LeafData::from).collect();
    (ModelRc::new(VecModel::from(slint_segments)), ModelRc::new(VecModel::from(slint_leaves)))
}