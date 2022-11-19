use bevy::{math::Vec3Swizzles, prelude::*};
use kurbo::{Affine, Point};
use piet_gpu::{
    PicoSvg, PietGpuRenderContext, RenderContext, Renderer, Text, TextAttribute, TextLayoutBuilder,
};

use piet_gpu_hal::{
    CmdBuf, Error, ImageLayout, Instance, QueryPool, Semaphore, Session, SubmittedCmdBuf, Swapchain,
};

use crate::math;

const NUM_FRAMES: usize = 2;

pub enum RenderType {
    Text(String, GlobalTransform),
    // Svg(PicoSvg, GlobalTransform, Vec2),
}

pub enum RenderLayer {
    Background,
    Middle,
    Foreground,
}

pub struct RenderCommand {
    render_type: RenderType,
    render_layer: RenderLayer,
}

impl RenderCommand {
    pub fn new(render_type: RenderType, render_layer: RenderLayer) -> Self {
        Self {
            render_type,
            render_layer,
        }
    }
}
pub struct RenderFrame {
    pub current_frame: usize,
}

pub struct RenderResources {
    present_semaphores: Vec<Semaphore>,
    query_pools: Vec<QueryPool>,
    cmd_bufs: [Option<CmdBuf>; NUM_FRAMES],
    submitted: [Option<piet_gpu_hal::SubmittedCmdBuf>; NUM_FRAMES],
    session: Session,
    swapchain: Swapchain,
    renderer: Renderer,
}

pub fn setup_piet_renderer(app_world: &World, render_app: &mut App) {
    let windows = app_world.get_resource::<Windows>().unwrap();
    let window = windows.get_primary().unwrap();

    let raw_window_handle = unsafe { window.raw_window_handle().get_handle() };
    let (instance, surface) = Instance::new(Some(&raw_window_handle), Default::default())
        .expect("Error: failed to creat Piet instance");
    let device = unsafe {
        instance
            .device(surface.as_ref())
            .expect("Error: Piet device creation failure")
    };
    let swapchain = unsafe {
        instance
            .swapchain(
                window.physical_width() as usize / 2,
                window.physical_height() as usize / 2,
                &device,
                surface.as_ref().unwrap(),
            )
            .unwrap()
    };
    let session = Session::new(device);

    let query_pools = (0..NUM_FRAMES)
        .map(|_| session.create_query_pool(8))
        .collect::<Result<Vec<_>, Error>>()
        .unwrap();
    let cmd_bufs: [Option<CmdBuf>; NUM_FRAMES] = Default::default();
    let submitted: [Option<SubmittedCmdBuf>; NUM_FRAMES] = Default::default();

    unsafe {
        let present_semaphores = (0..NUM_FRAMES)
            .map(|_| session.create_semaphore())
            .collect::<Result<Vec<_>, Error>>()
            .unwrap();

        let renderer = Renderer::new(
            &session,
            window.physical_width() as usize,
            window.physical_height() as usize,
            NUM_FRAMES,
        )
        .expect("Error: Piet renderer creation failure");

        render_app.insert_resource(RenderFrame { current_frame: 0 });

        render_app.insert_resource(PietGpuRenderContext::new());

        render_app.insert_non_send_resource(Some(RenderResources {
            present_semaphores,
            query_pools,
            cmd_bufs,
            submitted,
            session,
            swapchain,
            renderer,
        }));

        // Keep instance from being dropped
        render_app.insert_non_send_resource(instance);
    };
}

/// Prepare the render context by drawing elements to it in the order of their
/// respective render layers
pub fn prepare_frame(
    mut ctx: ResMut<PietGpuRenderContext>,
    mut events: EventReader<RenderCommand>,
) {
    let events: Vec<&RenderCommand> = events.iter().collect();
    for &command in events
        .iter()
        .filter(|c| matches!(c.render_layer, RenderLayer::Background { .. }))
    {
        execute_render_command(&mut ctx, command);
    }
    for &command in events
        .iter()
        .filter(|c| matches!(c.render_layer, RenderLayer::Middle { .. }))
    {
        execute_render_command(&mut ctx, command);
    }
    for &command in events
        .iter()
        .filter(|c| matches!(c.render_layer, RenderLayer::Foreground { .. }))
    {
        execute_render_command(&mut ctx, command);
    }
}

/// Draw an element to the render context according to the render command
fn execute_render_command(rc: &mut PietGpuRenderContext, command: &RenderCommand) {
    match &command.render_type {
        RenderType::Text(text, trans) => render_text(rc, text, *trans),
        // RenderType::Svg(svg, trans, center) => render_svg(svg, rc, *trans, *center),
    }
}

pub fn render_frame(
    mut renderer_res: NonSendMut<Option<RenderResources>>,
    mut frame: ResMut<RenderFrame>,
    mut ctx: ResMut<PietGpuRenderContext>,
) {
    unsafe {
        let RenderResources {
            present_semaphores,
            query_pools,
            mut cmd_bufs,
            mut submitted,
            session,
            mut swapchain,
            mut renderer,
        } = renderer_res.take().unwrap();

        let frame_idx = frame.current_frame % NUM_FRAMES;

        if let Some(submitted) = submitted[frame_idx].take() {
            cmd_bufs[frame_idx] = submitted.wait().unwrap();
            let _ts = session.fetch_query_pool(&query_pools[frame_idx]).unwrap();
        }

        if let Err(e) = renderer.upload_render_ctx(&mut ctx, frame_idx) {
            println!("error in uploading: {}", e);
        }
        *ctx = PietGpuRenderContext::new();

        let (image_idx, acquisition_semaphore) = swapchain.next().unwrap();
        let swap_image = swapchain.image(image_idx);
        let mut cmd_buf = cmd_bufs[frame_idx]
            .take()
            .unwrap_or_else(|| session.cmd_buf().unwrap());
        cmd_buf.begin();
        renderer.record(&mut cmd_buf, &query_pools[frame_idx], frame_idx);

        // Image -> Swapchain
        cmd_buf.image_barrier(&swap_image, ImageLayout::Undefined, ImageLayout::BlitDst);
        cmd_buf.blit_image(&renderer.image_dev, &swap_image);
        cmd_buf.image_barrier(&swap_image, ImageLayout::BlitDst, ImageLayout::Present);
        cmd_buf.finish();

        submitted[frame_idx] = Some(
            session
                .run_cmd_buf(
                    cmd_buf,
                    &[&acquisition_semaphore],
                    &[&present_semaphores[frame_idx]],
                )
                .unwrap(),
        );

        swapchain
            .present(image_idx, &[&present_semaphores[frame_idx]])
            .unwrap();

        frame.current_frame += 1;

        *renderer_res = Some(RenderResources {
            present_semaphores,
            query_pools,
            cmd_bufs,
            submitted,
            session,
            swapchain,
            renderer,
        });
    }
}

// pub fn render_svg(
//     svg: &PicoSvg,
//     rc: &mut PietGpuRenderContext,
//     transform: GlobalTransform,
//     center: Vec2,
// ) {
//     let trans = kurbo::Vec2::new(
//         transform.translation.x as f64,
//         transform.translation.y as f64,
//     );
//     let rotation_z = transform.rotation.to_euler(EulerRot::XYZ).2;

//     rc.save().unwrap();
//     rc.transform(
//         Affine::translate(trans)
//             * math::affine_scale_around(transform.scale.xy(), center)
//             * math::affine_rotate_around(rotation_z, center),
//     );
//     svg.render(rc);
//     rc.restore().unwrap();
// }

pub fn render_text(rc: &mut PietGpuRenderContext, text: &str, transform: GlobalTransform) {
    let layout = rc
        .text()
        .new_text_layout(text.to_string())
        .default_attribute(TextAttribute::FontSize(40.0))
        .build()
        .unwrap();
    rc.draw_text(
        &layout,
        Point::new(
            transform.translation.x.into(),
            transform.translation.y.into(),
        ),
    );
}
