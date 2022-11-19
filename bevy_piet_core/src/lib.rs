use bevy::{app::PluginGroupBuilder, prelude::*};

use bevy_piet_render::PietRenderPlugin;
use bevy_piet_text::PietTextPlugin;
// use bevy_piet_vector::PietVectorPlugin;

pub struct BevyPietPlugins;

impl PluginGroup for BevyPietPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(PietRenderPlugin::default())
            .add(PietTextPlugin::default())
        // .add(PietVectorPlugin::default())
    }
}
