use std::{cmp::Ordering, f64::consts::PI};

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_piet_render::RenderWorld;
use kurbo::{Affine, BezPath, Circle, Line, Point, Rect, Shape};

use piet_gpu::{PicoSvg, PietGpuRenderContext, RenderContext};

use crate::vector_image::{ExtractedVecImgInstances, VectorImageRenderAssets};

pub fn prepare_vector_images(
    mut extracted_app_world_vecs: ResMut<ExtractedVecImgInstances>,
    vec_images: Res<VectorImageRenderAssets>,
    mut ctx: ResMut<PietGpuRenderContext>,
) {
    // Sort sprites by z for correct transparency and then by handle to improve batching
    extracted_app_world_vecs.instances.sort_unstable_by(|a, b| {
        match a
            .transform
            .translation
            .z
            .partial_cmp(&b.transform.translation.z)
        {
            Some(Ordering::Equal) | None => a.vec_image_handle_id.cmp(&b.vec_image_handle_id),
            Some(other) => other,
        }
    });

    for extracted_inst in extracted_app_world_vecs.instances.iter() {
        if let Some(vec_image) = vec_images.get(&Handle::weak(extracted_inst.vec_image_handle_id)) {
            render_svg(&vec_image.svg, &mut ctx, extracted_inst.transform);
        }
    }
}

pub fn render_svg(svg: &PicoSvg, rc: &mut PietGpuRenderContext, transform: GlobalTransform) {
    let trans = kurbo::Vec2::new(
        transform.translation.x as f64,
        transform.translation.y as f64,
    );
    let rotation = transform.rotation.x as f64 * PI * 2.0;
    rc.transform(
        Affine::translate(trans)
            * Affine::rotate(rotation)
            * Affine::scale_non_uniform(transform.scale.x.into(), transform.scale.y.into()),
    );
    println!("rot={}", rotation);
    svg.render(rc);
}
