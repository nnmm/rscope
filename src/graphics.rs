use std::sync::mpsc::*;

use gfx;
use glutin;
use gfx_window_glutin;
use gfx::{Adapter, Bind, CommandQueue, Device, FrameSync, GraphicsPoolExt,
          Slice, Surface, SwapChain, SwapChainExt, WindowExt};
use gfx::traits::DeviceExt;
use gfx::buffer;

use parseopts::*;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        start: [f32; 2] = "aStart",
        end: [f32; 2] = "aEnd",
        idx: u32 = "aIdx",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

const CLEAR_COLOR: [f32; 4] = [0.04, 0.04, 0.04, 1.0];

#[derive(Debug)]
struct Vertices<A> {
    idx: u32,
    start: [f32; 2],
    end: [f32; 2],
    iter: A,
}

impl<A: Iterator<Item = (f32, f32)>> Iterator for Vertices<A> {
    type Item = Vertex;

    fn next(&mut self) -> Option<Vertex> {
        let v = Vertex { start: self.start, end: self.end, idx: self.idx };
        self.idx += 1;
        if self.idx % 4 == 0 {
            match self.iter.next() {
                None => { None }
                Some(p) => {
                    self.start = self.end;
                    self.end = [p.0, p.1];
                    Some(v)
                }
            }
        } else {
            Some(v)
        }
    }
}

fn lines<I: Iterator<Item = (f32, f32)>> (mut it: I) -> Vertices<I> {
    match it.next() {
        // stops immediately because of idx == 3
        None => { Vertices { idx: 3, start: [0.0, 0.0], end: [0.0, 0.0], iter: it } }
        Some(p) => { Vertices { idx: 0, start: [0.0, 0.0], end: [p.0, p.1], iter: it } }
    }
}

pub fn run_graphics(o : OscOpts, rcv: Receiver<Vec<(f32, f32)>>) {
    let OscOpts { magnification: mag, samples_per_frame: spf } = o;
    let num_verts = spf as u32 * 4;
    let quad_indices : Vec<u32> = (0..num_verts).map(|i| 4*(i/6) + (i%3) + (i%6)/3).collect();
    let mut vertices = lines(rcv.into_iter().flat_map(|v| v));

    let fake_vertices : Vec<_> = vec![
        Vertex { start: [ -0.5, -0.5 ], end: [ -0.5,  0.5 ], idx: 0 },
        Vertex { start: [ -0.5, -0.5 ], end: [ -0.5,  0.5 ], idx: 1 },
        Vertex { start: [ -0.5, -0.5 ], end: [ -0.5,  0.5 ], idx: 2 },
        Vertex { start: [ -0.5, -0.5 ], end: [ -0.5,  0.5 ], idx: 3 },
        Vertex { start: [ -0.5,  0.5 ], end: [  0.0,  0.5 ], idx: 4 },
        Vertex { start: [ -0.5,  0.5 ], end: [  0.0,  0.5 ], idx: 5 },
        Vertex { start: [ -0.5,  0.5 ], end: [  0.0,  0.5 ], idx: 6 },
        Vertex { start: [ -0.5,  0.5 ], end: [  0.0,  0.5 ], idx: 7 },
        Vertex { start: [  0.0,  0.5 ], end: [  0.5, -0.5 ], idx: 8 },
        Vertex { start: [  0.0,  0.5 ], end: [  0.5, -0.5 ], idx: 9 },
        Vertex { start: [  0.0,  0.5 ], end: [  0.5, -0.5 ], idx: 10 },
        Vertex { start: [  0.0,  0.5 ], end: [  0.5, -0.5 ], idx: 11 }
    ].into_iter().cycle().take(spf*4).collect();

    // graphics boilerplate
    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new()
        .with_title("rscope".to_string())
        .with_dimensions(1024, 768);
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
                    .with_color::<ColorFormat>();
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
            encoder.update_buffer(&data.vbuf, &fake_vertices, 0).expect("Could not update buffer.");
            encoder.draw(&slice, &pso, &data);
            encoder.synced_flush(&mut graphics_queue, &[&frame_semaphore], &[&draw_semaphore], Some(&frame_fence))
                   .expect("Could not flush encoder");
        }

        swap_chain.present(&mut graphics_queue, &[&draw_semaphore]);
        device.wait_for_fences(&[&frame_fence], gfx::WaitFor::All, 1_000_000);
        graphics_queue.cleanup();
        graphics_pool.reset();
    }
}