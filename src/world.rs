use std::collections::LinkedList;

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
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn step(&mut self) {
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

    pub fn set_cell(&mut self, x: i64, y: i64, value: bool) {
        let chunk_x = x.div_euclid(32);
        let chunk_y = y.div_euclid(32);
        let local_x = x.rem_euclid(32);
        let local_y = y.rem_euclid(32);
        let chunk_coords = pack_coords(chunk_x as i32, chunk_y as i32);

        // get or insert the chunk
        let chunk = self.chunks.entry(chunk_coords).or_insert(Chunk::new());
        let row = chunk.get_row(local_y as usize);
        let mask = 1 << (31 - local_x);
        let new_row = if value { row | mask } else { row & !mask };
        chunk.set_row(local_y as usize, new_row);
    }

    pub fn is_alive(&self, x: i64, y: i64) -> bool {
        let chunk_x = x.div_euclid(32);
        let chunk_y = y.div_euclid(32);
        let local_x = x.rem_euclid(32);
        let local_y = y.rem_euclid(32);
        let chunk_coords = pack_coords(chunk_x as i32, chunk_y as i32);
        let chunk = self.chunks.get(&chunk_coords);
        if let Some(chunk) = chunk {
            let row = chunk.get_row(local_y as usize);
            (row & (1 << (31 - local_x))) != 0
        } else {
            false
        }
    }

    pub fn live_cells(&self) -> impl Iterator<Item = (i64, i64)> {
        let mut list = LinkedList::new();
        for (chunk_coords, chunk) in &self.chunks {
            let (chunk_x, chunk_y) = unpack_coords(*chunk_coords);
            for local_y in 0..32 {
                let row = chunk.get_row(local_y as usize);
                if row == 0 {
                    continue;
                }
                for local_x in 0..32 {
                    if (row & (1 << (31 - local_x))) != 0 {
                        let x = chunk_x as i64 * 32 + local_x as i64;
                        let y = chunk_y as i64 * 32 + local_y as i64;
                        list.push_back((x, y));
                    }
                }
            }
        }
        list.into_iter()
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn load_pattern(&mut self, origin_x: i64, origin_y: i64, pattern: &str) {
        for (line_idx, line) in pattern.lines().enumerate() {
            for (char_idx, char) in line.chars().enumerate() {
                if char == '*' {
                    let x = origin_x + char_idx as i64;
                    let y = origin_y + line_idx as i64;
                    self.set_cell(x, y, true);
                }
            }
        }
    }

    fn get_neighbor_chunks(&self, coords: u64) -> NeighborChunks<'_> {
        let (x, y) = unpack_coords(coords);
        let get_chunk = |nx, ny| {
            let neighbor_coords = pack_coords(nx, ny);
            self.chunks.get(&neighbor_coords)
        };
        NeighborChunks {
            se: get_chunk(x + 1, y + 1),
            s: get_chunk(x, y + 1),
            sw: get_chunk(x - 1, y + 1),
            e: get_chunk(x + 1, y),
            w: get_chunk(x - 1, y),
            ne: get_chunk(x + 1, y - 1),
            n: get_chunk(x, y - 1),
            nw: get_chunk(x - 1, y - 1),
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
            set.insert(pack_coords(x, y - 1));
        }
        // -- NW --
        if (first_row >> 31) != 0 && neighbors.nw.is_none() {
            set.insert(pack_coords(x - 1, y - 1));
        }
        // -- NE --
        if (first_row & 1) != 0 && neighbors.ne.is_none() {
            set.insert(pack_coords(x + 1, y - 1));
        }
    }
    if last_row != 0 {
        // -- S --
        if neighbors.s.is_none() {
            set.insert(pack_coords(x, y + 1));
        }
        // -- SW --
        if (last_row >> 31) != 0 && neighbors.sw.is_none() {
            set.insert(pack_coords(x - 1, y + 1));
        }
        // -- SE --
        if (last_row & 1) != 0 && neighbors.se.is_none() {
            set.insert(pack_coords(x + 1, y + 1));
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
