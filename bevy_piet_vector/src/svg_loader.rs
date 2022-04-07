use anyhow::Result;

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    utils::BoxedFuture,
};
use piet_gpu::PicoSvg;

use crate::vector_image::VectorImage;

#[derive(Default)]
pub struct SvgAssetLoader;

impl AssetLoader for SvgAssetLoader {
    fn load<'a>(
        &'a self,
        _bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut svg_src_path =
                load_context.path().to_string_lossy().to_string();
            svg_src_path =
                "assets/".to_string() + &svg_src_path.replace("\\", "/");

            let xml_str = std::fs::read_to_string(&svg_src_path).unwrap();
            let svg = PicoSvg::load(&xml_str, 1.0).unwrap();

            load_context
                .set_default_asset(LoadedAsset::new(VectorImage { svg }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}
