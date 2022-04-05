use std::cmp::Ordering;

use bevy::{
    asset::HandleId,
    prelude::*,
    reflect::TypeUuid,
    utils::{HashMap, HashSet},
};
use bevy_piet_render::RenderWorld;
use piet_gpu::{PicoSvg, PietGpuRenderContext};

#[derive(Clone, TypeUuid)]
#[uuid = "6ea26da6-6cf8-4ea2-9986-1d7bf6c17d6f"]
pub struct VectorImage {
    pub svg: PicoSvg,
}

/// All the data extracted from a vector image instance necessary to render.
#[derive(Clone, Copy)]
pub struct ExtractedVecImgInstance {
    pub transform: GlobalTransform,
    pub vec_image_handle_id: HandleId,
}

/// Resource for storing all the vector image instances extracted at the current frame.,
#[derive(Default)]
pub struct ExtractedVecImgInstances {
    pub instances: Vec<ExtractedVecImgInstance>,
}

/// Extract all vector image instances from the "app world" and copy them to the piet "render world".
pub fn extract_vec_img_instances(
    mut render_world: ResMut<RenderWorld>,
    vec_img_inst_query: Query<(&GlobalTransform, &Handle<VectorImage>)>,
) {
    let mut instances = Vec::new();
    for (inst_transform, inst_vec_img) in vec_img_inst_query.iter() {
        instances.push(ExtractedVecImgInstance {
            transform: *inst_transform,
            vec_image_handle_id: inst_vec_img.id,
        })
    }

    // Copy extracted instances to render world in order to prepare
    // all vector images extracted during this frame for rendering.
    render_world.insert_resource(ExtractedVecImgInstances { instances });
}

/// Stores all render data representations of VectorImageRenderAssets as long as they exist.
pub type VectorImageRenderAssets = HashMap<Handle<VectorImage>, VectorImage>;

/// This system extracts all crated or modified assets of the corresponding [`VectorImage`] type
/// into the piet "render world".
pub fn extract_vec_img_render_assets(
    mut render_world: ResMut<RenderWorld>,
    mut events: EventReader<AssetEvent<VectorImage>>,
    assets: Res<Assets<VectorImage>>,
) {
    let mut vec_image_render_assets = render_world
        .get_resource_mut::<VectorImageRenderAssets>()
        .unwrap();

    let mut new_assets = HashSet::default();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                new_assets.insert(handle);
            }
            AssetEvent::Removed { handle } => {
                new_assets.remove(handle);
                vec_image_render_assets.remove(&handle);
            }
        }
    }

    for handle in new_assets.drain() {
        if let Some(asset) = assets.get(handle) {
            vec_image_render_assets.insert(handle.clone_weak(), asset.clone());
        }
    }
}
