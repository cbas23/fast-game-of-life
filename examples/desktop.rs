use std::sync::mpsc::{TrySendError, sync_channel};
use std::{thread, time::Duration};

use fast_game_of_life::World;
use minifb::{Key, Window, WindowOptions};

// visible area in global cell coordinates and its pixel dimensions
const CELL_SIZE: usize = 4;
const VIEW_WIDTH: usize = 256;
const VIEW_HEIGHT: usize = 256;
const VIEW_X: i64 = -64;
const VIEW_Y: i64 = -64;
const BUFFER_WIDTH: usize = VIEW_WIDTH * CELL_SIZE;
const BUFFER_HEIGHT: usize = VIEW_HEIGHT * CELL_SIZE;
const STEP_DELAY: Duration = Duration::from_millis(10);

// hardcoded pattern loaded when the example starts
const CELL_PATTERN: &str = "********.*****...***......*******.*****";

fn main() {
    // keep at most one completed frame waiting for the renderer
    let (sender, receiver) = sync_channel::<Vec<u32>>(1);

    // run the simulation on a separate thread
    thread::spawn(move || {
        let mut world = World::new();
        world.load_pattern(32, 32, CELL_PATTERN);

        let mut frame_buffer = vec![0; BUFFER_WIDTH * BUFFER_HEIGHT];

        loop {
            // compute and rasterize the next generation
            world.step();
            rasterize_world(&world, &mut frame_buffer);

            // send the frame if the renderer is ready, otherwise drop it
            match sender.try_send(frame_buffer.clone()) {
                Ok(()) | Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => break,
            }

            // limit how quickly generations are computed
            thread::sleep(STEP_DELAY);
        }
    });

    // create the desktop window on the main thread
    let mut window = Window::new(
        "fast-game-of-life",
        BUFFER_WIDTH,
        BUFFER_HEIGHT,
        WindowOptions::default(),
    )
    .expect("window creation failed");
    window.set_target_fps(60);

    // display the newest completed frame until the window closes
    let mut display_buffer = vec![0; BUFFER_WIDTH * BUFFER_HEIGHT];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // consume every pending frame and keep only the newest one
        while let Ok(new_buffer) = receiver.try_recv() {
            display_buffer = new_buffer;
        }

        window
            .update_with_buffer(&display_buffer, BUFFER_WIDTH, BUFFER_HEIGHT)
            .expect("failed to update window buffer");
    }
}

fn rasterize_world(world: &World, buffer: &mut [u32]) {
    // clear the previous frame to black
    buffer.fill(0x00000000);

    for (x, y) in world.live_cells() {
        // translate global cell coordinates into viewport coordinates
        let view_x = x - VIEW_X;
        let view_y = y - VIEW_Y;

        // skip cells outside the visible area
        if !(0..VIEW_WIDTH as i64).contains(&view_x) || !(0..VIEW_HEIGHT as i64).contains(&view_y) {
            continue;
        }

        // convert the cell position into the top-left pixel of its square
        let pixel_x = view_x as usize * CELL_SIZE;
        let pixel_y = view_y as usize * CELL_SIZE;

        // fill the cell's square with white pixels
        for offset_y in 0..CELL_SIZE {
            let row_start = (pixel_y + offset_y) * BUFFER_WIDTH + pixel_x;
            buffer[row_start..row_start + CELL_SIZE].fill(0x00ffffff);
        }
    }
}
