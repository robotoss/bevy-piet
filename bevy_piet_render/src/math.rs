// use bevy::prelude::*;
// use kurbo::Affine;

// pub fn affine_rotate_around(th: f32, p: Vec2) -> Affine {
//     let p = kurbo::Vec2::new(p.x.into(), p.y.into());
//     Affine::translate(p)
//         * Affine::rotate(th.into())
//         * Affine::translate(p * -1.0)
// }

// pub fn affine_scale_around(s: Vec2, p: Vec2) -> Affine {
//     let p = kurbo::Vec2::new(p.x.into(), p.y.into());
//     Affine::translate(p)
//         * Affine::scale_non_uniform(s.x.into(), s.y.into())
//         * Affine::translate(p * -1.0)
// }
