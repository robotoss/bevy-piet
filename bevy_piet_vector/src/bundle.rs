use crate::vector_image::VectorImage;
use bevy::prelude::{Bundle, Component, GlobalTransform, Handle, Transform};

#[derive(Default, Component, Clone)]
pub struct VectorImageInstance;

#[derive(Default, Bundle, Clone)]
pub struct VecImgInstanceBundle {
    pub vec_img_instance: VectorImageInstance,
    pub vector_image: Handle<VectorImage>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
