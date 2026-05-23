use std::{array, slice::Iter};

#[derive(Debug)]
pub struct Chunk {
    rows: [u32; 32],
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk { rows: [0; 32] }
    }

    pub fn get_row(&self, i: usize) -> u32 {
        if i > 32 {
            0
        } else {
            let val = self.rows[i];
            val
        }
    }

    pub fn get_first_row(&self) -> u32 {
        self.rows[0]
    }

    pub fn get_last_row(&self) -> u32 {
        self.rows[31]
    }

    pub fn set_row(&mut self, i: usize, v: u32) {
        if i > 32 {
            return;
        }
        self.rows[i] = v;
    }

    pub fn iter(&self) -> Iter<'_, u32> {
        self.rows.iter()
    }

    pub fn to_string_compact(&self) -> String {
        let mut str = String::new();
        let rows = &self.rows;
        for i in 0..16 {
            let upper = i * 2;
            let lower = i * 2 + 1;
            for j in (0..32).rev() {
                let upper_value = rows[upper] >> j & 1;
                let lower_value = rows[lower] >> j & 1;
                if upper_value == 1 {
                    if lower_value == 1 {
                        str.push('█');
                    } else {
                        str.push('▀');
                    }
                } else {
                    if lower_value == 1 {
                        str.push('▄');
                    } else {
                        str.push(' ');
                    }
                }
            }
            str.push('\n');
        }
        str
    }

    pub fn to_string_list_compact(&self) -> [String; 16] {
        let mut list: [String; 16] = array::from_fn(|_| String::new());
        let rows = &self.rows;
        for i in 0..16 {
            let upper = i * 2;
            let lower = i * 2 + 1;
            for j in (0..32).rev() {
                let upper_value = rows[upper] >> j & 1;
                let lower_value = rows[lower] >> j & 1;
                if upper_value == 1 {
                    if lower_value == 1 {
                        list[i].push('█');
                    } else {
                        list[i].push('▀');
                    }
                } else {
                    if lower_value == 1 {
                        list[i].push('▄');
                    } else {
                        list[i].push(' ');
                    }
                }
            }
        }
        list
    }

    pub fn from_string(string: &str) -> Chunk {
        let mut chunk = Chunk::new();
        let rows: Vec<&str> = string.split('\n').collect();
        for (r, row) in rows.iter().enumerate() {
            for (c, sym) in row.chars().enumerate().take(32) {
                if sym == '*' {
                    let mask = 1 << (31 - c);
                    chunk.rows[r] |= mask;
                }
            }
        }
        chunk
    }
}
