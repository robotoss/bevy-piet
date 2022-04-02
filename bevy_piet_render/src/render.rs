use std::thread;

use bevy::{prelude::*, ecs::world::WorldCell, ecs::event::Events };
use crate::RenderWorld;
use piet_gpu::{Renderer, test_scenes, PietGpuRenderContext, Blend};
use piet_gpu_hal::{Instance, Session, Semaphore, QueryPool, CmdBuf, SubmittedCmdBuf, Error, Swapchain, ImageLayout};
use piet_gpu::BlendMode::Normal;
use piet_gpu::CompositionMode::SrcOver;

const NUM_FRAMES: usize = 2;

pub struct RenderContext {
    pub ctx: PietGpuRenderContext,
}

pub struct RenderFrame {
    pub first: bool,
    pub current_frame: usize,
}

pub struct RenderResources {
    present_semaphores: Vec<Semaphore>,
    query_pools: Vec<QueryPool>,
    cmd_bufs: [Option<CmdBuf>; NUM_FRAMES],
    submitted: [Option<piet_gpu_hal::SubmittedCmdBuf>; NUM_FRAMES],
    session: Session,
    swapchain: Swapchain,
}
pub fn setup_piet_renderer(app_world: &World, render_app: &mut App) {
    let windows = app_world.get_resource::<Windows>().unwrap();
    let window = windows.get_primary().unwrap();

    let raw_window_handle = unsafe {window.raw_window_handle().get_handle()};
    let (instance, surface) = Instance::new(Some(&raw_window_handle), Default::default()).expect("Error: failed to creat Piet instance");
    let device = unsafe {instance.device(surface.as_ref()).expect("Error: Piet device creation failure")};
    let swapchain = unsafe {instance.swapchain(window.physical_width() as usize / 2, window.physical_height() as usize / 2, &device, surface.as_ref().unwrap()).unwrap()};
    let session = Session::new(device);
    
    unsafe {
        let present_semaphores = (0..NUM_FRAMES)
            .map(|_| session.create_semaphore())
            .collect::<Result<Vec<_>, Error>>().unwrap();
        let query_pools = (0..NUM_FRAMES)
            .map(|_| session.create_query_pool(8))
            .collect::<Result<Vec<_>, Error>>().unwrap();
        let cmd_bufs: [Option<CmdBuf>; NUM_FRAMES] = Default::default();
        let submitted: [Option<SubmittedCmdBuf>; NUM_FRAMES] = Default::default();

        render_app.insert_non_send_resource(present_semaphores);
        render_app.insert_non_send_resource(query_pools);
        render_app.insert_non_send_resource(cmd_bufs);
        render_app.insert_non_send_resource(submitted);
        render_app.insert_non_send_resource(session.clone());
        render_app.insert_non_send_resource(swapchain);
    };

    let renderer = unsafe {Renderer::new(&session, window.physical_width() as usize, window.physical_height() as usize, NUM_FRAMES).expect("Error: Piet renderer creation failure")};
    render_app.insert_non_send_resource(renderer);
    render_app.insert_resource(RenderContext {
        ctx: PietGpuRenderContext::new(),
    });
    render_app.insert_resource(RenderFrame {
        current_frame: 0,
        first: true,
    });
    
    println!("!!!!!!!!!!!!!!!!!!! win={:?}", window.raw_window_handle());
    
}

// // Forward window redraw events from app world to render world events
// pub fn extract_redraw_events(
//     mut events: EventReader<WindowRedrawRequested>,
//     mut render_world: ResMut<RenderWorld>,
// ) {
//     for event in events.iter() {
//         let mut redraw_events = render_world.get_resource_mut::<Events<RenderFrameEvent>>().unwrap();
//         redraw_events.send(RenderFrameEvent{ });
//         println!("REDRAW REQUESTED");

//     }
// }

pub fn prepare_frame(mut render_ctx: ResMut<RenderContext>, mut frame: ResMut<RenderFrame>) {
    // test_scenes::render_svg_anim(&mut render_ctx.ctx, "assets/Ghostscript_Tiger.svg", 1.0, frame.current_frame);
    test_scenes::render_blend_test(&mut render_ctx.ctx, frame.current_frame, Blend::new(Normal, SrcOver));
}

pub fn render_frame(
    // mut events: EventReader<RenderFrameEvent>,
    mut renderer: NonSendMut<Renderer>,
    mut frame: ResMut<RenderFrame>,
    mut cmd_bufs: NonSendMut<[Option<CmdBuf>; NUM_FRAMES]>,
    mut submitted: NonSendMut<[Option<piet_gpu_hal::SubmittedCmdBuf>; NUM_FRAMES]>,
    mut swapchain: NonSendMut<Swapchain>,
    query_pools: NonSendMut<Vec<QueryPool>>,
    present_semaphores: NonSendMut<Vec<Semaphore>>,
    session: NonSendMut<Session>,
) {
    unsafe {
        let frame_idx = frame.current_frame % NUM_FRAMES;

        if let Some(submitted) = submitted[frame_idx].take() {
            cmd_bufs[frame_idx] = submitted.wait().unwrap();
            let ts = session.fetch_query_pool(&query_pools[frame_idx]).unwrap();
            if !ts.is_empty() {

            }
        }

        let mut ctx = PietGpuRenderContext::new();

        use piet_gpu::{Blend, BlendMode::*, CompositionMode::*};

        test_scenes::render_blend_test(&mut ctx, frame.current_frame, Blend::new(Normal, SrcOver));
        // test_scenes::render_tiger(&mut ctx, current_frame);
        // test_scenes::render_svg_anim(&mut ctx, "piet-gpu/Ghostscript_Tiger.svg", 1., current_frame);

        if let Err(e) = renderer.upload_render_ctx(&mut ctx, frame_idx) {
            println!("error in uploading: {}", e);
        }

        let (image_idx, acquisition_semaphore) = swapchain.next().unwrap();
        let swap_image = swapchain.image(image_idx);
        let mut cmd_buf = cmd_bufs[frame_idx].take().unwrap_or_else(|| session.cmd_buf().unwrap());
        cmd_buf.begin();
        renderer.record(&mut cmd_buf, &query_pools[frame_idx], frame_idx);

        // Image -> Swapchain
        cmd_buf.image_barrier(
            &swap_image,
            ImageLayout::Undefined,
            ImageLayout::BlitDst,
        );
        cmd_buf.blit_image(&renderer.image_dev, &swap_image);
        cmd_buf.image_barrier(&swap_image, ImageLayout::BlitDst, ImageLayout::Present);
        cmd_buf.finish();

        submitted[frame_idx] = Some(session
            .run_cmd_buf(
                cmd_buf,
                &[&acquisition_semaphore],
                &[&present_semaphores[frame_idx]],
            )
            .unwrap());

        swapchain
            .present(image_idx, &[&present_semaphores[frame_idx]])
            .unwrap();

        frame.current_frame += 1;

    }
}