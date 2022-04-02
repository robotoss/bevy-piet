use bevy::{app::PluginGroupBuilder, prelude::*};

use bevy_piet_render::PietRenderPlugin;

pub struct BevyPietPlugins;

impl PluginGroup for BevyPietPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(PietRenderPlugin::default());
    }
}
