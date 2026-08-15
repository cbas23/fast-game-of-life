use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::Duration;

use fast_game_of_life::World;

// visible area in global cell coordinates
const VIEW_X: i64 = -24;
const VIEW_Y: i64 = -12;
const VIEW_WIDTH: usize = 49;
const VIEW_HEIGHT: usize = 25;
const FRAME_DELAY: Duration = Duration::from_millis(125);

// hardcoded glider loaded when the example starts
const START_CELLS: &str = "
    .*.
    ..*
    ***
";

fn main() -> io::Result<()> {
    // create the world and load the starting pattern
    let mut world = World::new();
    world.load_pattern(-6, -6, START_CELLS);

    // buffer terminal writes and detect whether the output is interactive
    let stdout = io::stdout();
    let interactive = stdout.is_terminal();
    let mut output = io::BufWriter::new(stdout.lock());
    let mut generation = 0_u64;

    // clear the terminal once before drawing the first frame
    if interactive {
        write!(output, "\x1b[2J")?;
    }

    loop {
        // move the cursor back to the top instead of printing a new frame below
        if interactive {
            write!(output, "\x1b[H")?;
        }

        // draw the current generation and send it to the terminal
        render(&world, generation, &mut output)?;
        output.flush()?;

        // a redirected invocation renders one frame and exits, which makes the
        // example convenient to use in scripts and automated checks.
        if !interactive {
            break;
        }

        // compute the next generation and limit the animation speed
        world.step();
        generation += 1;
        thread::sleep(FRAME_DELAY);
    }

    Ok(())
}

fn render(world: &World, generation: u64, output: &mut impl Write) -> io::Result<()> {
    // create a screen-sized buffer and copy visible live cells into it
    let mut cells = vec![false; VIEW_WIDTH * VIEW_HEIGHT];

    for (x, y) in world.live_cells() {
        // translate global cell coordinates into viewport coordinates
        let view_x = x - VIEW_X;
        let view_y = y - VIEW_Y;

        // ignore cells outside the visible area
        if (0..VIEW_WIDTH as i64).contains(&view_x) && (0..VIEW_HEIGHT as i64).contains(&view_y) {
            cells[view_y as usize * VIEW_WIDTH + view_x as usize] = true;
        }
    }

    // draw the status line and top border
    writeln!(
        output,
        "Fast Game of Life | generation {generation} | Ctrl+C to quit"
    )?;
    write!(output, "┌")?;
    for _ in 0..VIEW_WIDTH {
        write!(output, "──")?;
    }
    writeln!(output, "┐")?;

    // use two terminal columns per cell to keep cells roughly square
    for row in cells.chunks_exact(VIEW_WIDTH) {
        write!(output, "│")?;
        for &alive in row {
            write!(output, "{}", if alive { "██" } else { "  " })?;
        }
        writeln!(output, "│")?;
    }

    // draw the bottom border
    write!(output, "└")?;
    for _ in 0..VIEW_WIDTH {
        write!(output, "──")?;
    }
    writeln!(output, "┘")
}
