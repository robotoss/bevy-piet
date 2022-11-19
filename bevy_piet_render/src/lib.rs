use std::ops::{Deref, DerefMut};

mod math;
mod render;

use bevy::{
    app::{App, AppLabel, Plugin},
    ecs::{event::Events, schedule::ShouldRun},
    prelude::*,
};
use render::{prepare_frame, render_frame, setup_piet_renderer};

/// A Label for the rendering sub-app.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, AppLabel)]
pub struct PietRenderApp;

pub use render::{RenderCommand, RenderLayer, RenderType};

/// The Render App World. This is only available as a resource during the
/// Extract step.
#[derive(Default)]
pub struct RenderWorld(World);

impl Deref for RenderWorld {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderWorld {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A "scratch" world used to avoid allocating new worlds every frame when
/// swapping out the [`RenderWorld`].
#[derive(Default)]
struct ScratchRenderWorld(World);

/// Contains the Bevy interface to the Piet renderer.
#[derive(Default)]
pub struct PietRenderPlugin;

/// The labels of the default App rendering stages.
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum PietRenderStage {
    Setup,

    /// Extract data from the "app world" and insert it into the "render
    /// world". This step should be kept as short as possible to increase
    /// the "pipelining potential" for running the next frame while
    /// rendering the current frame.
    Extract,

    /// Prepare render resources from the extracted data for the GPU.
    Prepare,

    /// Actual rendering happens here.
    /// In most cases, only the render backend should insert resources here.
    Render,

    /// Cleanup render resources here.
    Cleanup,
}

impl Plugin for PietRenderPlugin {
    /// Initializes the renderer, sets up the
    /// [`PietRenderStage`](PietRenderStage) and creates the rendering sub-app.
    fn build(&self, app: &mut App) {
        app.init_resource::<ScratchRenderWorld>();

        let mut render_app = App::empty();

        render_app
            .add_stage(
                PietRenderStage::Setup,
                SystemStage::parallel().with_run_criteria(ShouldRun::once), // .with_system(setup_piet.exclusive_system().at_start()),
            )
            .add_stage(
                PietRenderStage::Extract,
                SystemStage::parallel(), // .with_system(extract_redraw_events),
            )
            .add_stage(
                PietRenderStage::Prepare,
                SystemStage::parallel().with_system(prepare_frame), // .with_system(Events::<RenderFrameEvent>::update_system),
            )
            .add_stage(PietRenderStage::Render, SystemStage::single(render_frame))
            .add_stage(PietRenderStage::Cleanup, SystemStage::parallel())
            .init_resource::<Events<RenderCommand>>()
            .add_system_to_stage(
                PietRenderStage::Prepare,
                Events::<RenderCommand>::update_system,
            );

        setup_piet_renderer(&app.world, &mut render_app);

        app.add_sub_app(PietRenderApp, render_app, move |app_world, render_app| {
            #[cfg(feature = "trace")]
            let render_span = bevy_utils::tracing::info_span!("renderer subapp");
            #[cfg(feature = "trace")]
            let _render_guard = render_span.enter();
            {
                #[cfg(feature = "trace")]
                let stage_span =
                    bevy_utils::tracing::info_span!("stage", name = "reserve_and_flush");
                #[cfg(feature = "trace")]
                let _stage_guard = stage_span.enter();

                // reserve all existing app entities for use in render_app
                // they can only be spawned using `get_or_spawn()`
                let meta_len = app_world.entities().meta.len();
                render_app
                    .world
                    .entities()
                    .reserve_entities(meta_len as u32);

                // flushing as "invalid" ensures that app world entities
                // aren't added as "empty archetype" entities by default
                // these entities cannot be accessed without spawning
                // directly onto them this _only_ works
                // as expected because clear_entities() is called at the end
                // of every frame.
                render_app.world.entities_mut().flush_as_invalid();
            }

            {
                let setup = render_app
                    .schedule
                    .get_stage_mut::<SystemStage>(&PietRenderStage::Setup)
                    .unwrap();
                setup.run(&mut render_app.world);
            }

            {
                #[cfg(feature = "trace")]
                let stage_span = bevy_utils::tracing::info_span!("stage", name = "extract");
                #[cfg(feature = "trace")]
                let _stage_guard = stage_span.enter();

                // extract
                extract(app_world, render_app);
            }

            {
                #[cfg(feature = "trace")]
                let stage_span = bevy_utils::tracing::info_span!("stage", name = "prepare");
                #[cfg(feature = "trace")]
                let _stage_guard = stage_span.enter();

                // prepare
                let prepare = render_app
                    .schedule
                    .get_stage_mut::<SystemStage>(&PietRenderStage::Prepare)
                    .unwrap();
                prepare.run(&mut render_app.world);
            }

            {
                #[cfg(feature = "trace")]
                let stage_span = bevy_utils::tracing::info_span!("stage", name = "render");
                #[cfg(feature = "trace")]
                let _stage_guard = stage_span.enter();

                // render
                let render = render_app
                    .schedule
                    .get_stage_mut::<SystemStage>(&PietRenderStage::Render)
                    .unwrap();
                render.run(&mut render_app.world);
            }

            {
                #[cfg(feature = "trace")]
                let stage_span = bevy_utils::tracing::info_span!("stage", name = "cleanup");
                #[cfg(feature = "trace")]
                let _stage_guard = stage_span.enter();

                // cleanup
                let cleanup = render_app
                    .schedule
                    .get_stage_mut::<SystemStage>(&PietRenderStage::Cleanup)
                    .unwrap();
                cleanup.run(&mut render_app.world);

                render_app.world.clear_entities();
            }
        });
    }
}

/// Executes the [`Extract`](PietRenderStage::Extract) stage of the renderer.
/// This updates the render world with the extracted ECS data of the current
/// frame.
fn extract(app_world: &mut World, render_app: &mut App) {
    let extract = render_app
        .schedule
        .get_stage_mut::<SystemStage>(&PietRenderStage::Extract)
        .unwrap();

    // temporarily add the render world to the app world as a resource
    let scratch_world = app_world.remove_resource::<ScratchRenderWorld>().unwrap();
    let render_world = std::mem::replace(&mut render_app.world, scratch_world.0);
    app_world.insert_resource(RenderWorld(render_world));

    extract.run(app_world);

    // add the render world back to the render app
    let render_world = app_world.remove_resource::<RenderWorld>().unwrap();
    let scratch_world = std::mem::replace(&mut render_app.world, render_world.0);
    app_world.insert_resource(ScratchRenderWorld(scratch_world));

    extract.apply_buffers(&mut render_app.world);
}
