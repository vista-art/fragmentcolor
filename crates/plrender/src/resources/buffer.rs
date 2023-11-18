use crate::math::Quad;
use std::mem::size_of;

// Based off wgpu example 'capture'
#[derive(Debug, Clone)]
pub struct BufferSize {
    pub width: usize,
    pub height: usize,
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: u32,
}

impl BufferSize {
    pub fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = (unpadded_bytes_per_row + padded_bytes_per_row_padding) as u32;

        Self {
            width,
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }

    pub fn size(&self) -> u64 {
        self.padded_bytes_per_row as u64 * self.height as u64
    }
}

#[derive(Debug)]
pub struct Buffer {
    pub size: BufferSize,
    pub buffer: wgpu::Buffer,
}

#[derive(Debug)]
pub struct TextureBuffer {
    pub buffer: Buffer,
    pub clip_region: Quad,
}
