use std::{cmp::Ordering, f64::consts::PI};

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_piet_render::RenderWorld;
use kurbo::{Affine, BezPath, Circle, Line, Point, Rect, Shape};

use piet_gpu::{PicoSvg, PietGpuRenderContext, RenderContext, Text, TextAttribute, TextLayoutBuilder};

use crate::bundle::TextLabel;

pub struct ExtractedTextLabel {
    text: String,
    transform: GlobalTransform,
}

pub struct ExtractedTextLabels {
    text_labels: Vec<ExtractedTextLabel>,
}

pub fn extract_text_labels(
    query: Query<(&TextLabel, &GlobalTransform)>,
    mut render_world: ResMut<RenderWorld>,
) {
    let mut text_labels = Vec::new();
    for (text_label, transform) in query.iter() {
        text_labels.push(ExtractedTextLabel {
            text: text_label.text.clone(),
            transform: *transform,
        })

    }

    // Copy extracted instances to render world in order to prepare
    // all vector images extracted during this frame for rendering.
    render_world.insert_resource(ExtractedTextLabels { text_labels });
}

pub fn prepare_text_labels(
    extracted: ResMut<ExtractedTextLabels>,
    // vec_images: Res<VectorImageRenderAssets>,
    mut ctx: ResMut<PietGpuRenderContext>,
) {
    for text_label in extracted.text_labels.iter() {
        render_text(&mut ctx, &text_label.text, text_label.transform.translation.xy());
    }
    
}

pub fn render_text(
    rc: &mut PietGpuRenderContext,
    text: &str,
    translation: Vec2,
) {

    let layout = rc
        .text()
        .new_text_layout(text.to_string())
        .default_attribute(TextAttribute::FontSize(40.0))
        .build()
        .unwrap();
    rc.draw_text(&layout, Point::new(translation.x.into(), translation.y.into()));

}