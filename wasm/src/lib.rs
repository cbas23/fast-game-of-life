use fast_game_of_life::World;
use wasm_bindgen::prelude::*;

// every cell coordinate backed by an i32 chunk coordinate fits in this range
const MIN_CELL_COORDINATE: i64 = i32::MIN as i64 * 32;
const MAX_CELL_COORDINATE: i64 = i32::MAX as i64 * 32 + 31;

// JavaScript-facing wrapper around the platform-independent simulation
#[wasm_bindgen]
pub struct WasmWorld {
    inner: World,
}

#[wasm_bindgen]
impl WasmWorld {
    // create an empty world
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: World::new(),
        }
    }

    // compute one generation
    pub fn step(&mut self) {
        self.inner.step();
    }

    // compute multiple generations without crossing the JavaScript boundary
    pub fn step_by(&mut self, generations: u32) {
        for _ in 0..generations {
            self.inner.step();
        }
    }

    // set one cell using JavaScript number coordinates
    pub fn set_cell(&mut self, x: f64, y: f64, alive: bool) -> Result<(), JsValue> {
        let x = parse_coordinate("x", x)?;
        let y = parse_coordinate("y", y)?;
        self.inner.set_cell(x, y, alive);
        Ok(())
    }

    // check whether one cell is alive
    pub fn is_alive(&self, x: f64, y: f64) -> Result<bool, JsValue> {
        let x = parse_coordinate("x", x)?;
        let y = parse_coordinate("y", y)?;
        Ok(self.inner.is_alive(x, y))
    }

    // load a text pattern at a global cell coordinate
    pub fn load_pattern(&mut self, x: f64, y: f64, pattern: &str) -> Result<(), JsValue> {
        let x = parse_coordinate("x", x)?;
        let y = parse_coordinate("y", y)?;

        // make sure the complete pattern remains inside the supported world
        let width = pattern.lines().map(|line| line.chars().count()).max();
        let height = pattern.lines().count();
        validate_pattern_extent("x", x, width)?;
        validate_pattern_extent("y", y, Some(height))?;

        self.inner.load_pattern(x, y, pattern);
        Ok(())
    }

    // return [x0, y0, x1, y1, ...] as a JavaScript Float64Array
    pub fn live_cells(&self) -> Vec<f64> {
        self.inner
            .live_cells()
            .flat_map(|(x, y)| [x as f64, y as f64])
            .collect()
    }

    // remove every live cell
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl Default for WasmWorld {
    fn default() -> Self {
        Self::new()
    }
}

// reject values that cannot be represented safely by the chunk coordinate key
fn parse_coordinate(name: &str, value: f64) -> Result<i64, JsValue> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Err(JsValue::from_str(&format!(
            "{name} must be a finite integer"
        )));
    }

    if value < MIN_CELL_COORDINATE as f64 || value > MAX_CELL_COORDINATE as f64 {
        return Err(JsValue::from_str(&format!(
            "{name} must be between {MIN_CELL_COORDINATE} and {MAX_CELL_COORDINATE}"
        )));
    }

    Ok(value as i64)
}

// check the last coordinate touched by a pattern before loading it
fn validate_pattern_extent(name: &str, origin: i64, length: Option<usize>) -> Result<(), JsValue> {
    let Some(length) = length.filter(|&length| length > 0) else {
        return Ok(());
    };

    let last = origin
        .checked_add(length as i64 - 1)
        .ok_or_else(|| JsValue::from_str("pattern coordinates overflowed"))?;

    if last > MAX_CELL_COORDINATE {
        return Err(JsValue::from_str(&format!(
            "pattern exceeds the maximum {name} coordinate"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::WasmWorld;

    #[test]
    fn wraps_the_world_api() {
        let mut world = WasmWorld::new();
        world.load_pattern(0.0, 0.0, "***").unwrap();
        world.step();

        assert!(world.is_alive(1.0, -1.0).unwrap());
        assert!(world.is_alive(1.0, 0.0).unwrap());
        assert!(world.is_alive(1.0, 1.0).unwrap());
        assert_eq!(world.live_cells().len(), 6);
    }
}
