use bevy::{app::PluginGroupBuilder, prelude::*};

use bevy_piet_render::PietRenderPlugin;
use bevy_piet_vector::PietVectorPlugin;
use bevy_piet_text::PietTextPlugin;

pub struct BevyPietPlugins;

impl PluginGroup for BevyPietPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(PietRenderPlugin::default());
        group.add(PietVectorPlugin::default());
        group.add(PietTextPlugin::default());
    }
}
