use bevy::prelude::*;
use bevy_piet_render::{PietRenderApp, PietRenderStage};
use render::prepare_vector_images;
use svg_loader::SvgAssetLoader;
use vector_image::{ExtractedVecImgInstances, extract_vec_img_instances, extract_vec_img_render_assets, VectorImageRenderAssets, VectorImage};

mod bundle;
mod vector_image;
mod svg_loader;
mod render;

pub use bundle::{VecImgInstanceBundle, VectorImageInstance};

#[derive(Default)]
pub struct PietVectorPlugin;

impl Plugin for PietVectorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_asset::<VectorImage>()
            .init_asset_loader::<SvgAssetLoader>();
        if let Ok(render_app) = app.get_sub_app_mut(PietRenderApp) {
            render_app
                .init_resource::<ExtractedVecImgInstances>()
                .init_resource::<VectorImageRenderAssets>()
                .add_system_to_stage(PietRenderStage::Extract, extract_vec_img_render_assets)
                .add_system_to_stage(PietRenderStage::Extract, extract_vec_img_instances)
                .add_system_to_stage(PietRenderStage::Prepare, prepare_vector_images);
        }
    }
}
