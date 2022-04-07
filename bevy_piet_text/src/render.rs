use bevy::prelude::*;
use bevy_piet_render::{RenderCommand, RenderLayer, RenderType, RenderWorld};

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
    extracted_text_labels: ResMut<ExtractedTextLabels>,
    mut render_commands: EventWriter<RenderCommand>,
) {
    for extracted in extracted_text_labels.text_labels.iter() {
        let render_command =
            RenderType::Text(extracted.text.clone(), extracted.transform);
        render_commands
            .send(RenderCommand::new(render_command, RenderLayer::Foreground));
        // render_text(&mut ctx, &text_label.text,
        // text_label.transform.translation.xy());
    }
}
