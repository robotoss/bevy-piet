use bevy::prelude::*;

#[derive(Default, Component, Clone)]
pub struct TextLabel {
    pub text: String,
}

#[derive(Default, Bundle, Clone)]
pub struct TextLabelBundle {
    pub text_label: TextLabel,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
