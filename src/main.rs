use std::{thread, time::Duration};

use crate::{chunk::Chunk, world::World};
use indoc::indoc;
use minifb::{Key, Window, WindowOptions};
use std::sync::mpsc::{TrySendError, sync_channel};

const CELL_SIZE: usize = 3; // zoom: 1 cell = 2x2 pixels
const CHUNK_DIM: usize = 32;
const VIEW_CHUNKS_X: usize = 8;
const VIEW_CHUNKS_Y: usize = 8;
const BUF_W: usize = VIEW_CHUNKS_X * CHUNK_DIM * CELL_SIZE;
const BUF_H: usize = VIEW_CHUNKS_Y * CHUNK_DIM * CELL_SIZE;
const DELAY: u64 = 50;

const BORDER_COLOR: u32 = 0x00333333;

const CELL_PATTERN: &str = indoc! {"
   	...***......
    ...*........
    ....*.......
    *...........
    *.*.......**
    **.......*.*
    ...........*
    ...*........
    ....*.......
    ..***.......
"};

mod chunk;
mod world;

fn main() {
    let (tx, rx) = sync_channel::<Vec<u32>>(1); // backpressure: max 1 pending frame
    // === Simulation Thread ===
    thread::spawn(move || {
        let mut world = World::new();
        //  load your starting pattern into world
        world.add_chunk(0, 0, Chunk::from_string(CELL_PATTERN));

        let mut frame_buf = vec![0u32; BUF_W * BUF_H];
        loop {
            // Run generations as fast as possible
            world.compute_gen();
            // Rasterize the current sparse grid into the pixel buffer
            rasterize_world(&world, &mut frame_buf, -2, -2);
            // Push to renderer; if the renderer hasn't consumed the previous
            // frame yet, drop it and replace (i.e., decoupled update/draw).
            match tx.try_send(frame_buf.clone()) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => {} // renderer lagging, skip frame
                Err(TrySendError::Disconnected(_)) => break,
            }

            thread::sleep(Duration::from_millis(DELAY));
        }
    });
    // === Render Thread (Main) ===
    let mut window = Window::new("fast-game-of-life", BUF_W, BUF_H, WindowOptions::default())
        .expect("Window creation failed");
    window.set_target_fps(60);
    let mut display_buf = vec![0u32; BUF_W * BUF_H];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Grab the latest completed frame, if any
        while let Ok(new_buf) = rx.try_recv() {
            display_buf = new_buf;
        }
        window
            .update_with_buffer(&display_buf, BUF_W, BUF_H)
            .unwrap();
    }
}

fn set_pixel(buf: &mut [u32], x: isize, y: isize, color: u32) {
    if x >= 0 && x < BUF_W as isize && y >= 0 && y < BUF_H as isize {
        buf[(y as usize) * BUF_W + (x as usize)] = color;
    }
}

fn rasterize_world(world: &World, buf: &mut [u32], origin_cx: i32, origin_cy: i32) {
    buf.fill(0x00000000); // black background
    for cy in origin_cy..origin_cy + VIEW_CHUNKS_Y as i32 {
        for cx in origin_cx..origin_cx + VIEW_CHUNKS_X as i32 {
            if let Some(chunk) = world.get_chunk(cx, cy) {
                let base_px = ((cx - origin_cx) as usize * CHUNK_DIM * CELL_SIZE) as isize;
                let base_py = ((VIEW_CHUNKS_Y - 1 - (cy - origin_cy) as usize)
                    * CHUNK_DIM
                    * CELL_SIZE) as isize;
                for row in 0..CHUNK_DIM {
                    let bits = chunk.get_row(row);
                    // Your chunk packs bit 31 as the leftmost column
                    for col in 0..CHUNK_DIM {
                        let alive = (bits >> (31 - col)) & 1;
                        if alive == 0 {
                            continue;
                        }
                        let px = base_px + (col * CELL_SIZE) as isize;
                        let py = base_py + (row * CELL_SIZE) as isize;
                        // Fill CELL_SIZE x CELL_SIZE block
                        for dy in 0..CELL_SIZE as isize {
                            for dx in 0..CELL_SIZE as isize {
                                let x = px + dx;
                                let y = py + dy;
                                if x >= 0 && x < BUF_W as isize && y >= 0 && y < BUF_H as isize {
                                    buf[(y as usize) * BUF_W + (x as usize)] = 0x00FFFFFF;
                                }
                            }
                        }
                    }
                }
                // Draw chunk borders
                let chunk_px = (CHUNK_DIM * CELL_SIZE) as isize;
                for i in 0..chunk_px {
                    set_pixel(buf, base_px + i, base_py, BORDER_COLOR); // bottom
                    set_pixel(buf, base_px + i, base_py + chunk_px - 1, BORDER_COLOR); // top
                    set_pixel(buf, base_px, base_py + i, BORDER_COLOR); // left
                    set_pixel(buf, base_px + chunk_px - 1, base_py + i, BORDER_COLOR); // right
                }
            }
        }
    }
}
