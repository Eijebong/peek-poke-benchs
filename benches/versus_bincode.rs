use serde::{Deserialize, Serialize};

use criterion::{black_box, criterion_group, criterion_main, Benchmark, Criterion};

use bincode::{decode_from_slice, encode_into_slice};
use peek_poke::{PeekPoke, Poke};
use std::{io, ptr};

#[derive(Debug, Default, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Default, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub struct Rect {
    pub point: Point,
    pub size: Size,
}

pub type PipelineSourceId = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub struct PipelineId(pub PipelineSourceId, pub u32);

impl Default for PipelineId {
    fn default() -> Self {
        PipelineId(0, 0)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub struct ClipChainId(pub u64, pub PipelineId);

#[repr(C)]
#[derive(Clone, Copy, Debug, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub enum ClipId {
    Clip(usize, PipelineId),
    ClipChain(ClipChainId),
}

impl Default for ClipId {
    fn default() -> Self {
        ClipId::Clip(!0, PipelineId::default())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub struct ItemTag(u64, u16);

#[repr(C)]
#[derive(Debug, Default, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub struct SpatialId(pub usize, PipelineId);

#[repr(C)]
#[derive(Debug, Default, bincode::Encode, bincode::Decode, Deserialize, PartialEq, PeekPoke, Serialize)]
pub struct CommonItemProperties {
    pub clip_rect: Rect,
    pub spatial_id: SpatialId,
    pub clip_id: ClipId,
    pub hit_info: Option<ItemTag>,
    pub is_backface_visible: bool,
}

// This is used by webrender_api
#[derive(Clone, Copy)]
struct UnsafeReader {
    start: *const u8,
    end: *const u8,
}

impl UnsafeReader {
    #[inline(always)]
    fn new(buf: &[u8]) -> UnsafeReader {
        unsafe {
            let end = buf.as_ptr().add(buf.len());
            let start = buf.as_ptr();
            UnsafeReader { start, end }
        }
    }

    // This read implementation is significantly faster than the standard &[u8] one.
    //
    // First, it only supports reading exactly buf.len() bytes. This ensures that
    // the argument to memcpy is always buf.len() and will allow a constant buf.len()
    // to be propagated through to memcpy which LLVM will turn into explicit loads and
    // stores. The standard implementation does a len = min(slice.len(), buf.len())
    //
    // Second, we only need to adjust 'start' after reading and it's only adjusted by a
    // constant. This allows LLVM to avoid adjusting the length field after ever read
    // and lets it be aggregated into a single adjustment.
    #[inline(always)]
    fn read_internal(&mut self, buf: &mut [u8]) {
        // this is safe because we panic if start + buf.len() > end
        unsafe {
            assert!(
                self.start.add(buf.len()) <= self.end,
                "UnsafeReader: read past end of target"
            );
            ptr::copy_nonoverlapping(self.start, buf.as_mut_ptr(), buf.len());
            self.start = self.start.add(buf.len());
        }
    }
}

impl io::Read for UnsafeReader {
    // These methods were not being inlined and we need them to be so that the memcpy
    // is for a constant size
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_internal(buf);
        Ok(buf.len())
    }
    #[inline(always)]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.read_internal(buf);
        Ok(())
    }
}

#[allow(unused_must_use)]
fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
        "struct::serialize",
        Benchmark::new("peek_poke::poke_into", |b| {
            let mut buffer = Vec::with_capacity(1024);
            let ptr = buffer.as_mut_ptr();
            b.iter(|| {
                let my_struct = CommonItemProperties {
                    clip_rect: Rect {
                        point: Point { x: 1.0, y: 2.0 },
                        size: Size { w: 4.0, h: 5.0 },
                    },
                    clip_id: ClipId::Clip(5, PipelineId(1, 2)),
                    spatial_id: SpatialId(3, PipelineId(4, 5)),
                    hit_info: None,
                    is_backface_visible: true,
                };
                black_box(unsafe { black_box(&my_struct).poke_into(ptr) });
            })
        })
        .with_function("bincode::serialize", |b| {
            //let mut buffer = Vec::with_capacity(1024);
            let mut buffer = vec![0u8;1024];
            let config = bincode::config::Configuration::legacy();
            b.iter(|| {
                //buffer.clear();
                let my_struct = CommonItemProperties {
                    clip_rect: Rect {
                        point: Point { x: 1.0, y: 2.0 },
                        size: Size { w: 4.0, h: 5.0 },
                    },
                    clip_id: ClipId::Clip(5, PipelineId(1, 2)),
                    spatial_id: SpatialId(3, PipelineId(4, 5)),
                    hit_info: None,
                    is_backface_visible: true,
                };
                black_box(bincode::encode_into_slice(black_box(&my_struct), &mut buffer.as_mut_slice(), config).unwrap());
            })
        }),
    );

    c.bench(
        "struct::deserialize",
        Benchmark::new("peek_poke::peek_from", |b| {
            let bytes = vec![
                0u8, 0, 128, 63, 0, 0, 0, 64, 0, 0, 128, 64, 0, 0, 160, 64, 3, 0, 0, 0, 0, 0, 0, 0,
                4, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 1,
            ];
            b.iter(|| {
                black_box(unsafe {
                    peek_poke::peek_from_default::<CommonItemProperties>(black_box(bytes.as_ptr()))
                });
            })
        })
        .with_function("bincode::deserialize", |b| {
            let config = bincode::config::Configuration::legacy();
            let bytes = vec![
                0u8, 0, 128, 63, 0, 0, 0, 64, 0, 0, 128, 64, 0, 0, 160, 64, 3, 0, 0, 0, 0, 0, 0, 0,
                4, 0, 0, 0, 5, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 0, 1,
            ];
            let mut result = CommonItemProperties::default();
            let reader = UnsafeReader::new(&bytes);
            b.iter(|| {
                black_box(decode_from_slice::<CommonItemProperties, _>(&bytes, config));
            })
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
