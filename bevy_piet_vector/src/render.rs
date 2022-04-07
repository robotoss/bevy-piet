use std::cmp::Ordering;

use bevy::prelude::*;
use bevy_piet_render::{RenderCommand, RenderLayer, RenderType};

use piet_gpu::PietGpuRenderContext;

use crate::vector_image::{ExtractedVecImgInstances, VectorImageRenderAssets};

pub fn prepare_vector_images(
    mut extracted_app_world_vecs: ResMut<ExtractedVecImgInstances>,
    vec_images: Res<VectorImageRenderAssets>,
    mut render_commands: EventWriter<RenderCommand>,
    _ctx: ResMut<PietGpuRenderContext>,
) {
    // Sort images by z for correct transparency and then by handle to improve
    // batching
    extracted_app_world_vecs.instances.sort_unstable_by(|a, b| {
        match a
            .transform
            .translation
            .z
            .partial_cmp(&b.transform.translation.z)
        {
            Some(Ordering::Equal) | None => {
                a.vec_image_handle_id.cmp(&b.vec_image_handle_id)
            }
            Some(other) => other,
        }
    });

    for extracted in extracted_app_world_vecs.instances.iter() {
        if let Some(vec_image) =
            vec_images.get(&Handle::weak(extracted.vec_image_handle_id))
        {
            let render_command = RenderType::Svg(
                vec_image.svg.clone(),
                extracted.transform,
                extracted.vec_image_inst.center,
            );
            render_commands
                .send(RenderCommand::new(render_command, RenderLayer::Middle))
            // render_svg(
            //     &vec_image.svg,
            //     &mut ctx,
            //     extracted_inst.transform,
            //     extracted_inst.vec_image_inst.center,
            // );
        }
    }
}
