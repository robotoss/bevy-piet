use bevy::prelude::*;
use bevy_piet_render::{PietRenderApp, PietRenderStage};
use render::{extract_text_labels, prepare_text_labels};

mod bundle;
mod render;

pub use bundle::{TextLabel, TextLabelBundle};

#[derive(Default)]
pub struct PietTextPlugin;

impl Plugin for PietTextPlugin {
    fn build(&self, app: &mut App) {
        // app.add_asset::<VectorImage>()
        //     .init_asset_loader::<SvgAssetLoader>();
        if let Ok(render_app) = app.get_sub_app_mut(PietRenderApp) {
            render_app
                .add_system_to_stage(
                    PietRenderStage::Extract,
                    extract_text_labels,
                )
                .add_system_to_stage(
                    PietRenderStage::Prepare,
                    prepare_text_labels,
                );
            //     .init_resource::<ExtractedVecImgInstances>()
            //     .init_resource::<VectorImageRenderAssets>()
            // .add_system_to_stage(
            //     PietRenderStage::Extract,
            //     extract_vec_img_instances,
        }
    }
}
