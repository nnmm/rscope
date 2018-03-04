use std::sync::mpsc::*;

use chan;
use gfx;
use gfx::buffer;
use gfx_window_glutin;
use gfx::{Adapter, Bind, CommandQueue, Device, FrameSync, GraphicsPoolExt,
          Slice, Surface, SwapChain, SwapChainExt, WindowExt};
use gfx::traits::DeviceExt;
use glutin;

use parseopts::*;
use pipeline::*;


const CLEAR_COLOR: [f32; 4] = [0.01, 0.01, 0.01, 1.0];

pub fn run_graphics(o : OscOpts, rcv: Receiver<Vec<(f32, f32)>>, _sdone: chan::Sender<()>) {
    let OscOpts { magnification: mag, samples_per_frame: spf } = o;
    let num_verts = spf as u32 * 4;
    let quad_indices : Vec<u32> = (0..num_verts).map(|i| 4*(i/6) + (i%3) + (i%6)/3).collect();
    let mut vertices = lines(rcv.into_iter().flat_map(|v| v));

    // graphics boilerplate
    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new()
        .with_title("rscope".to_string())
        .with_dimensions(800, 800);
    let gl_builder = glutin::ContextBuilder::new().with_vsync(true);
    let window = glutin::GlWindow::new(wb, gl_builder, &events_loop).unwrap();

    // Acquire surface and adapters
    let (mut surface, adapters) = gfx_window_glutin::Window::new(window).get_surface_and_adapters();

    // Open gpu (device and queues)
    let gfx::Gpu { mut device, mut graphics_queues, .. } =
        adapters[0].open_with(|family, ty| {
            ((ty.supports_graphics() && surface.supports_queue(&family)) as u32, gfx::QueueType::Graphics)
        });
    let mut graphics_queue = graphics_queues.pop().expect("Unable to find a graphics queue.");

    // Create swapchain
    let config = gfx::SwapchainConfig::new()
                    .with_color::<gfx::format::Rgba8>();
    let mut swap_chain = surface.build_swapchain(config, &graphics_queue);
    let views = swap_chain.create_color_views(&mut device);

    let pso = device.create_pipeline_simple(
        include_bytes!("shader/line_150.glslv"),
        include_bytes!("shader/line_150.glslf"),
        pipe::new()
    ).unwrap();
    let mut graphics_pool = graphics_queue.create_graphics_pool(1);
    let frame_semaphore = device.create_semaphore();
    let draw_semaphore = device.create_semaphore();
    let frame_fence = device.create_fence(false);

    // let (vertex_buffer, slice) = device.create_vertex_buffer_with_slice(&frame_verts, &quad_indices);
    let vertex_buffer = device.create_buffer(spf*4,
                                             buffer::Role::Vertex,
                                             gfx::memory::Usage::Dynamic,
                                             Bind::empty())
                              .expect("Unable to create vertex buffer.");
    let slice = Slice {
        start: 0,
        end: num_verts,
        base_vertex: 0,
        instances: None,
        buffer: device.create_index_buffer(quad_indices.as_slice()),
    };
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: views[0].clone(),
    };

    // main loop
    let mut running = true;
    while running {
        // fetch events
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::Closed => running = false,
                    glutin::WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::Escape), ..
                        }, ..
                    } => return,
                    glutin::WindowEvent::Resized(_width, _height) => {
                        // TODO
                    },
                    _ => (),
                }
            }
        });

        // Get next frame
        let frame = swap_chain.acquire_frame(FrameSync::Semaphore(&frame_semaphore));
        data.out = views[frame.id()].clone();
        let frame_verts : Vec<_> = vertices.by_ref().take(spf*4).collect();

        // draw a frame
        // wait for frame -> draw -> signal -> present
        {
            let mut encoder = graphics_pool.acquire_graphics_encoder();
            encoder.clear(&data.out, CLEAR_COLOR);
            encoder.update_buffer(&data.vbuf, &frame_verts, 0).expect("Could not update buffer.");
            encoder.draw(&slice, &pso, &data);
            encoder.synced_flush(&mut graphics_queue, &[&frame_semaphore], &[&draw_semaphore], Some(&frame_fence))
                   .expect("Could not flush encoder");
        }

        swap_chain.present(&mut graphics_queue, &[&draw_semaphore]);
        device.wait_for_fences(&[&frame_fence], gfx::WaitFor::All, 1_000_000);
        graphics_queue.cleanup();
        graphics_pool.reset();
    }
    println!("bye");
}