//! Shows how to render simple primitive shapes with a single color.

use bevy::prelude::*;
use bevy_piet::BevyPietPlugins;
use bevy_piet_text::{TextLabel, TextLabelBundle};

#[derive(Component)]
struct Text;

#[derive(Component)]
struct Node(i32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyPietPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut command: Commands) {
    command
        .spawn(TextLabelBundle {
            text_label: TextLabel {
                text: "Hello world".to_string(),
            },
            transform: Transform::from_xyz(125.0, 160.0, 0.0).with_scale(Vec3::new(2.0, 2.0, 2.0)),
            ..default()
        })
        .insert(Node(-5));
}
