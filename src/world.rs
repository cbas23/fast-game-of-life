use std::{
    array,
    io::{self, Write},
    thread,
    time::Duration,
};

use hashbrown::HashMap;
use hashbrown::HashSet;

use crate::chunk::Chunk;

#[derive(Debug)]
pub struct World {
    chunks: HashMap<u64, Chunk>,
}

struct NeighborChunks<'a> {
    ne: Option<&'a Chunk>,
    e: Option<&'a Chunk>,
    se: Option<&'a Chunk>,
    n: Option<&'a Chunk>,
    s: Option<&'a Chunk>,
    nw: Option<&'a Chunk>,
    w: Option<&'a Chunk>,
    sw: Option<&'a Chunk>,
}

impl World {
    pub fn new() -> World {
        World {
            chunks: HashMap::new(),
        }
    }

    pub fn compute_gen(&mut self) {
        let mut new_gen: HashMap<u64, Chunk> = HashMap::new();
        let mut extra_coords: HashSet<u64> = HashSet::new();
        // go through all the chunks
        for (coords, chunk) in self.chunks.iter() {
            let neighbors = self.get_neighbor_chunks(*coords);
            add_neighbors_to_set(&mut extra_coords, *coords, chunk, &neighbors);
            let new_chunk = compute_chunk_gen(Some(chunk), &neighbors);
            if let Some(new_chunk) = new_chunk {
                new_gen.insert(*coords, new_chunk);
            }
        }
        // compute all the added chunks to the new generation
        for coords in extra_coords.iter() {
            let neighbors = self.get_neighbor_chunks(*coords);
            let new_chunk = compute_chunk_gen(None, &neighbors);
            if let Some(new_chunk) = new_chunk {
                new_gen.insert(*coords, new_chunk);
            }
        }
        // change the old_gen with the new_gen
        self.chunks = new_gen;
    }

    pub fn add_chunk(&mut self, x: i32, y: i32, chunk: Chunk) {
        let coords = pack_coords(x, y);
        self.chunks.insert(coords, chunk);
    }

    pub fn get_chunk(&self, x: i32, y: i32) -> Option<&Chunk> {
        let coords = pack_coords(x, y);
        self.chunks.get(&coords)
    }

    pub fn load_from_string(&mut self, chunk_x: i32, chunk_y: i32, s: &str) {
        
    }

    fn get_neighbor_chunks(&self, coords: u64) -> NeighborChunks<'_> {
        let (x, y) = unpack_coords(coords);
        let get_chunk = |nx, ny| {
            let neighbor_coords = pack_coords(nx, ny);
            self.chunks.get(&neighbor_coords)
        };
        NeighborChunks {
            ne: get_chunk(x + 1, y + 1),
            e: get_chunk(x + 1, y),
            se: get_chunk(x + 1, y - 1),
            n: get_chunk(x, y + 1),
            s: get_chunk(x, y - 1),
            nw: get_chunk(x - 1, y + 1),
            w: get_chunk(x - 1, y),
            sw: get_chunk(x - 1, y - 1),
        }
    }
}

fn compute_chunk_gen(opt_chunk: Option<&Chunk>, neighbors: &NeighborChunks) -> Option<Chunk> {
    let mut new_chunk = Chunk::new();
    let mut has_data = false;
    let mut bit_adder: [u32; 4];

    let chunk = if let Some(chunk) = opt_chunk {
        chunk
    } else {
        &Chunk::new()
    };

    // compute first row
    bit_adder = [0; 4];
    let masks = calc_top_row_masks(&neighbors, chunk);
    for mask in masks {
        add_to_bit_adder(&mut bit_adder, mask);
    }
    let new_row = calc_next_row(&bit_adder, chunk.get_row(0));
    new_chunk.set_row(0, new_row);
    has_data |= new_row != 0;

    // compute inner rows
    for i in 1..31 {
        bit_adder = [0; 4];
        let masks = calc_inner_row_masks(&neighbors, chunk, i);
        for mask in masks {
            add_to_bit_adder(&mut bit_adder, mask);
        }
        let new_row = calc_next_row(&bit_adder, chunk.get_row(i));
        new_chunk.set_row(i, new_row);
        has_data |= new_row != 0;
    }

    // compute last row
    bit_adder = [0; 4];
    let masks = calc_bottom_row_masks(&neighbors, chunk);
    for mask in masks {
        add_to_bit_adder(&mut bit_adder, mask);
    }
    let new_row = calc_next_row(&bit_adder, chunk.get_row(31));
    new_chunk.set_row(31, new_row);
    has_data |= new_row != 0;

    // after
    if has_data { Some(new_chunk) } else { None }
}

fn add_neighbors_to_set(
    set: &mut HashSet<u64>,
    coords: u64,
    chunk: &Chunk,
    neighbors: &NeighborChunks,
) {
    let first_row = chunk.get_first_row();
    let last_row = chunk.get_last_row();
    let (x, y) = unpack_coords(coords);

    if first_row != 0 {
        // -- N --
        if neighbors.n.is_none() {
            set.insert(pack_coords(x, y + 1));
        }
        // -- NW --
        if (first_row >> 31) != 0 && neighbors.nw.is_none() {
            set.insert(pack_coords(x - 1, y + 1));
        }
        // -- NE --
        if (first_row & 1) != 0 && neighbors.ne.is_none() {
            set.insert(pack_coords(x + 1, y + 1));
        }
    }
    if last_row != 0 {
        // -- S --
        if neighbors.s.is_none() {
            set.insert(pack_coords(x, y - 1));
        }
        // -- SW --
        if (last_row >> 31) != 0 && neighbors.sw.is_none() {
            set.insert(pack_coords(x - 1, y - 1));
        }
        // -- SE --
        if (last_row & 1) != 0 && neighbors.se.is_none() {
            set.insert(pack_coords(x + 1, y - 1));
        }
    }
    let mut left_sum: u32 = 0;
    let mut right_sum: u32 = 0;
    for row in chunk.iter() {
        left_sum += row >> 31;
        right_sum += row & 1;
    }
    // -- W --
    if left_sum != 0 && neighbors.w.is_none() {
        set.insert(pack_coords(x - 1, y));
    }
    // -- E --
    if right_sum != 0 && neighbors.e.is_none() {
        set.insert(pack_coords(x + 1, y));
    }
}

fn calc_top_row_masks(neighbors: &NeighborChunks, chunk: &Chunk) -> [u32; 8] {
    let north_row = get_chunk_row(neighbors.n, 31);
    let mid_row = get_chunk_row(Some(chunk), 0);
    let south_row = get_chunk_row(Some(chunk), 1);
    [
        shift_left_fill(north_row, get_chunk_row(neighbors.ne, 31)),
        shift_right_fill(north_row, get_chunk_row(neighbors.nw, 31)),
        north_row,
        shift_left_fill(mid_row, get_chunk_row(neighbors.e, 0)),
        shift_right_fill(mid_row, get_chunk_row(neighbors.w, 0)),
        south_row,
        shift_left_fill(south_row, get_chunk_row(neighbors.e, 1)),
        shift_right_fill(south_row, get_chunk_row(neighbors.w, 1)),
    ]
}

fn calc_inner_row_masks(neighbors: &NeighborChunks, chunk: &Chunk, i: usize) -> [u32; 8] {
    let north_row = get_chunk_row(Some(chunk), i - 1);
    let mid_row = get_chunk_row(Some(chunk), i);
    let south_row = get_chunk_row(Some(chunk), i + 1);
    [
        shift_left_fill(north_row, get_chunk_row(neighbors.e, i - 1)),
        shift_right_fill(north_row, get_chunk_row(neighbors.w, i - 1)),
        north_row,
        shift_left_fill(mid_row, get_chunk_row(neighbors.e, i)),
        shift_right_fill(mid_row, get_chunk_row(neighbors.w, i)),
        south_row,
        shift_left_fill(south_row, get_chunk_row(neighbors.e, i + 1)),
        shift_right_fill(south_row, get_chunk_row(neighbors.w, i + 1)),
    ]
}

fn calc_bottom_row_masks(neighbors: &NeighborChunks, chunk: &Chunk) -> [u32; 8] {
    let north_row = get_chunk_row(Some(chunk), 30);
    let mid_row = get_chunk_row(Some(chunk), 31);
    let south_row = get_chunk_row(neighbors.s, 0);
    [
        shift_left_fill(north_row, get_chunk_row(neighbors.e, 30)),
        shift_right_fill(north_row, get_chunk_row(neighbors.w, 30)),
        north_row,
        shift_left_fill(mid_row, get_chunk_row(neighbors.e, 31)),
        shift_right_fill(mid_row, get_chunk_row(neighbors.w, 31)),
        south_row,
        shift_left_fill(south_row, get_chunk_row(neighbors.se, 0)),
        shift_right_fill(south_row, get_chunk_row(neighbors.sw, 0)),
    ]
}

// turns [1,2,3][4,0,0] into [2,3,4]
fn shift_left_fill(base: u32, ext: u32) -> u32 {
    let base = base << 1;
    base | (ext >> 31)
}

// turn [1,2,3][0,0,4] into [4,1,2]
fn shift_right_fill(base: u32, ext: u32) -> u32 {
    let base = base >> 1;
    base | (ext << 31)
}

fn calc_next_row(bit_adder: &[u32; 4], row: u32) -> u32 {
    let is2 = !bit_adder[0] & bit_adder[1] & !bit_adder[2] & !bit_adder[3];
    let is3 = bit_adder[0] & bit_adder[1] & !bit_adder[2] & !bit_adder[3];
    is3 | (row & is2)
}

fn get_chunk_row(chunk: Option<&Chunk>, row: usize) -> u32 {
    if let Some(c) = chunk {
        c.get_row(row)
    } else {
        0
    }
}

fn add_to_bit_adder(bit_adder: &mut [u32; 4], mask: u32) {
    let carry1 = bit_adder[0] & mask;
    bit_adder[0] ^= mask;
    let carry2 = bit_adder[1] & carry1;
    bit_adder[1] ^= carry1;
    let carry3 = bit_adder[2] & carry2;
    bit_adder[2] ^= carry2;
    bit_adder[3] ^= carry3;
}

// stored as u64 from (x,y)
fn pack_coords(x: i32, y: i32) -> u64 {
    let x_part = (x as u32 as u64) << 32;
    let y_part = y as u32 as u64;
    x_part | y_part
}

// restored from u64 to (x,y)
fn unpack_coords(packed: u64) -> (i32, i32) {
    let x = (packed >> 32) as i32;
    let y = packed as i32;
    (x, y)
}

// ==========================================================================//
//                                TESTS                                      //
// ==========================================================================//

pub fn simple_test() {
    let mut w = World::new();

    let mut nw_chunk = Chunk::new();
    nw_chunk.set_row(31, 0x00000001);
    let mut n_chunk = Chunk::new();
    n_chunk.set_row(31, 0xF000000F);
    let mut ne_chunk = Chunk::new();
    ne_chunk.set_row(31, 0x80000000);

    let mut w_chunk = Chunk::new();
    w_chunk.set_row(0, 0x00000001);
    w_chunk.set_row(1, 0x00000001);
    w_chunk.set_row(2, 0x00000001);
    w_chunk.set_row(3, 0x00000001);
    w_chunk.set_row(28, 0x00000001);
    w_chunk.set_row(29, 0x00000001);
    w_chunk.set_row(30, 0x00000001);
    w_chunk.set_row(31, 0x00000001);
    let mut e_chunk = Chunk::new();
    e_chunk.set_row(0, 0x80000000);
    e_chunk.set_row(1, 0x80000000);
    e_chunk.set_row(2, 0x80000000);
    e_chunk.set_row(3, 0x80000000);
    e_chunk.set_row(28, 0x80000000);
    e_chunk.set_row(29, 0x80000000);
    e_chunk.set_row(30, 0x80000000);
    e_chunk.set_row(31, 0x80000000);

    let mut sw_chunk = Chunk::new();
    sw_chunk.set_row(0, 0x00000001);
    let mut s_chunk = Chunk::new();
    s_chunk.set_row(0, 0xF000000F);
    let mut se_chunk = Chunk::new();
    se_chunk.set_row(0, 0x80000000);

    let mut central_chunk = Chunk::new();
    central_chunk.set_row(2, 0x3000000C);
    central_chunk.set_row(3, 0x3000000C);
    central_chunk.set_row(28, 0x3000000C);
    central_chunk.set_row(29, 0x3000000C);

    w.add_chunk(-1, 1, nw_chunk);
    w.add_chunk(0, 1, n_chunk);
    w.add_chunk(1, 1, ne_chunk);
    w.add_chunk(-1, 0, w_chunk);
    w.add_chunk(1, 0, e_chunk);
    w.add_chunk(-1, -1, sw_chunk);
    w.add_chunk(0, -1, s_chunk);
    w.add_chunk(1, -1, se_chunk);
    w.add_chunk(0, 0, central_chunk);

    print!("\n");

    for (coord, chunk) in w.chunks.iter() {
        let (x, y) = unpack_coords(*coord);
        println!("\nCHUNK: ({}, {})", x, y);
        let chunk_lines = chunk.to_string_list_compact();
        for line in chunk_lines {
            print!("\x1b[40m");
            print!("{}", line);
            print!("\x1b[0m\n");
        }
    }

    println!("\n=== MASKS ===\n");

    let mut full_chunk_masks: [Vec<u32>; 8] = array::from_fn(|_| Vec::new());

    if let Some(chunk) = w.get_chunk(0, 0) {
        let neighbors = w.get_neighbor_chunks(pack_coords(0, 0));

        let upper_masks = calc_top_row_masks(&neighbors, chunk);
        for (i, mask) in upper_masks.iter().enumerate() {
            full_chunk_masks[i].push(*mask);
        }
        for k in 1..31 {
            let upper_masks = calc_inner_row_masks(&neighbors, chunk, k);
            for (i, mask) in upper_masks.iter().enumerate() {
                full_chunk_masks[i].push(*mask);
            }
        }
        let lower_masks = calc_bottom_row_masks(&neighbors, chunk);
        for (i, mask) in lower_masks.iter().enumerate() {
            full_chunk_masks[i].push(*mask);
        }
    };

    for (i, mask_rows) in full_chunk_masks.iter().enumerate() {
        let mut mask_chunk = Chunk::new();
        for (row_idx, &value) in mask_rows.iter().enumerate() {
            mask_chunk.set_row(row_idx, value);
        }
        println!("\nMASK CHUNK: {}", i);
        let chunk_lines = mask_chunk.to_string_list_compact();
        for line in chunk_lines {
            print!("\x1b[40m");
            print!("{}", line);
            print!("\x1b[0m\n");
        }
    }

    // display(&mut w, 2000);
}

fn display(w: &mut World, speed: u64) {
    let from = -1;
    let to = 1;
    let mut display: Vec<Vec<String>> = Vec::new();

    for i in 1..=200 {
        // clear the screen
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

        // set the display values
        for x in from..=to {
            let mut column: Vec<String> = Vec::new();
            for y in from..=to {
                if let Some(chunk) = w.get_chunk(x, y) {
                    let text_list = chunk.to_string_list_compact();
                    column.append(&mut text_list.to_vec());
                } else {
                    let emtpy_list: [String; 16] = array::from_fn(|_| String::from(".".repeat(32)));
                    column.append(&mut emtpy_list.to_vec());
                };
            }
            display.push(column);
        }

        // alternate bg 40 and 100

        print!("Frame: {}\n\n", i);

        let Some(first) = display.first() else {
            return;
        };

        let col_len = first.len();
        let len = display.len();

        println!("from {}, to {}", 0, col_len);

        for y in 0..=col_len {
            // println!("some col");
            for x in 0..=len {
                // println!("some row");
                if let Some(col) = display.get(x) {
                    if let Some(string) = col.get(y) {
                        let color_code = 31 + (x % 6);
                        print!("\x1b[{}m{}\x1b[0m", color_code, string);
                    };
                };
            }
            print!("\n");
        }

        display.clear();

        io::stdout().flush().unwrap();

        w.compute_gen();

        thread::sleep(Duration::from_millis(speed));
    }
}
