use crate::vector_image::VectorImage;
use bevy::prelude::*;

#[derive(Default, Component, Clone, Copy)]
pub struct VectorImageInstance {
    pub center: Vec2,
}

#[derive(Default, Bundle, Clone)]
pub struct VecImgInstanceBundle {
    pub vec_img_instance: VectorImageInstance,
    pub vector_image: Handle<VectorImage>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
