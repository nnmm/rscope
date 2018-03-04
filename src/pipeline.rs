use gfx;
use gfx::state::{Blend, BlendChannel, BlendValue, Equation, Factor};


const BLENDFUN: Blend = Blend {
    color: BlendChannel {
        equation: Equation::Add,
        source: Factor::ZeroPlus(BlendValue::SourceAlpha),
        destination: Factor::One,
    },
    alpha: BlendChannel {
        equation: Equation::Add,
        source: Factor::ZeroPlus(BlendValue::SourceAlpha),
        destination: Factor::One,
    },
};

gfx_defines!{
    vertex Vertex {
        start: [f32; 2] = "aStart",
        end: [f32; 2] = "aEnd",
        idx: u32 = "aIdx",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::BlendTarget<gfx::format::Rgba8> = ("Target0", gfx::state::ColorMask::all(), BLENDFUN),
    }
}

#[derive(Debug)]
pub struct Vertices<A> {
    idx: u32,
    start: [f32; 2],
    end: [f32; 2],
    iter: A,
}

impl<A: Iterator<Item = (f32, f32)>> Iterator for Vertices<A> {
    type Item = Vertex;

    fn next(&mut self) -> Option<Vertex> {
        let v = Vertex { start: self.start, end: self.end, idx: self.idx };
        self.idx = (self.idx + 1) % 4;
        if self.idx == 0 {
            match self.iter.next() {
                None => {
                    None
                }
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

pub fn lines<I: Iterator<Item = (f32, f32)>> (mut it: I) -> Vertices<I> {
    match it.next() {
        // stops immediately because of idx == 3
        None => { Vertices { idx: 3, start: [0.0, 0.0], end: [0.0, 0.0], iter: it } }
        Some(p) => { Vertices { idx: 0, start: [0.0, 0.0], end: [p.0, p.1], iter: it } }
    }
}