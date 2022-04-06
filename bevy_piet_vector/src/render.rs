use std::{cmp::Ordering, f64::consts::PI};

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_piet_render::RenderWorld;
use kurbo::{Affine, BezPath, Circle, Line, Point, Rect, Shape};

use piet_gpu::{PicoSvg, PietGpuRenderContext, RenderContext};

use crate::{
    math,
    vector_image::{ExtractedVecImgInstances, VectorImageRenderAssets},
};

pub fn prepare_vector_images(
    mut extracted_app_world_vecs: ResMut<ExtractedVecImgInstances>,
    vec_images: Res<VectorImageRenderAssets>,
    mut ctx: ResMut<PietGpuRenderContext>,
) {
    // Sort images by z for correct transparency and then by handle to improve batching
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

    for extracted_inst in extracted_app_world_vecs.instances.iter() {
        if let Some(vec_image) =
            vec_images.get(&Handle::weak(extracted_inst.vec_image_handle_id))
        {
            render_svg(
                &vec_image.svg,
                &mut ctx,
                extracted_inst.transform,
                extracted_inst.vec_image_inst.center,
            );
        }
    }
}

pub fn render_svg(
    svg: &PicoSvg,
    rc: &mut PietGpuRenderContext,
    transform: GlobalTransform,
    center: Vec2,
) {
    let trans = kurbo::Vec2::new(
        transform.translation.x as f64,
        transform.translation.y as f64,
    );
    let rotation_x = transform.rotation.to_euler(EulerRot::XYZ).0;

    rc.transform(
        Affine::translate(trans)
            * math::affine_scale_around(transform.scale.xy(), center)
            * math::affine_rotate_around(rotation_x, center),
    );
    // println!("rot={}", rotation / PI);
    svg.render(rc);
}
