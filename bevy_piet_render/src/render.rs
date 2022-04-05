use bevy::prelude::*;
use piet_gpu::{test_scenes, PietGpuRenderContext, Renderer};

use piet_gpu_hal::{
    CmdBuf, Error, ImageLayout, Instance, QueryPool, Semaphore, Session, SubmittedCmdBuf,
    Swapchain,
};

const NUM_FRAMES: usize = 2;

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

pub fn prepare_frame(ctx: ResMut<PietGpuRenderContext>, frame: Res<RenderFrame>) {
    let scale = 5.0 * (frame.current_frame as f64 / 200.0).sin();
    // test_scenes::render_svg(ctx.into_inner(), "assets/Ghostscript_Tiger.svg", scale);
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
            let ts = session.fetch_query_pool(&query_pools[frame_idx]).unwrap();
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
