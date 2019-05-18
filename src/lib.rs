mod utils;

use wasm_bindgen::prelude::*;
use std::{thread, time, iter};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
    Wood = 2,
    Fire = 3,
    Sand = 4
}


#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    nexts: Vec<Cell>
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbour_row = (row + delta_row) % self.height;
                let neighbour_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbour_row, neighbour_col);
                // count += self.cells[idx] as u8;
                if self.cells[idx] == Cell::Alive {
                    count += 1;
                }
            }
        }
        count
    }

    // Function that calculates the cell identity of a Sand object in the next tick
    fn sand_state_next_tick(&mut self, row : u32, column : u32) -> Cell {
        if self.sand_can_fall(row+1, column) {
            let bottom_index = self.get_index(row+1, column);
            self.nexts[bottom_index] = Cell::Sand;
            Cell::Dead
        } else{
            Cell::Sand
        }
    }

    // Function to see if a cell can have Sand fall into it (ie is Fire or Dead)
    fn sand_can_fall(&self, row : u32, column : u32) -> bool {
        // Checks if cell exists
        if row < 0 || row > self.height-1 || column < 0 || column > self.width-1 {
            return false;
        }

        // If so, checks if current cell is empty or fire
        let idx = self.get_index(row, column);
        let cell = self.cells[idx];
        match cell {
            Cell::Dead => true,
            Cell::Fire => true,
            _ => false
        }

    }

    fn wood_state_next_tick(&self, row : u32, column : u32) -> Cell {
        if self.is_fire_neighbour(row, column) {
            Cell::Fire // If a Wood cell is next to fire, it sets on fire
        } else {
            Cell::Wood // Otherwise, it remains wood
        }
    }

    fn fire_state_next_tick(&self, row: u32, column : u32) -> Cell {
        Cell::Dead // Fire isn't active for more than one tick
    }

    fn life_state_next_tick(&self, row : u32, column : u32, cell : Cell) -> Cell {
        let idx = self.get_index(row, column);
        let live_neighbours = self.live_neighbour_count(row, column);

        match (cell, live_neighbours) {
            // Rule 1: Any live cell with fewer than two live neighbours
            // dies, as if caused by underpopulation.
            (Cell::Alive, x) if x < 2 => Cell::Dead,
            // Rule 2: Any live cell with two or three live neighbours
            // lives on to the next generation.
            (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
            // Rule 3: Any live cell with more than three live
            // neighbours dies, as if by overpopulation.
            (Cell::Alive, x) if x > 3 => Cell::Dead,
            // Rule 4: Any dead cell with exactly three live neighbours
            // becomes a live cell, as if by reproduction.
            (Cell::Dead, 3) => Cell::Alive,
            // All other cells remain in the same state.
            (otherwise, _) => cell
        }
    }

    fn is_fire_neighbour(&self, row: u32, column: u32) -> bool {
        let top_fire = self.is_fire(row-1, column);
        let bottom_fire = self.is_fire(row+1, column);
        let left_fire = self.is_fire(row, column-1);
        let right_fire = self.is_fire(row, column+1);

        return top_fire || bottom_fire || left_fire || right_fire;
    }

    fn is_fire(&self, row : u32, column : u32) -> bool {
        // Checks if cell exists
        if row < 0 || row > self.height-1 || column < 0 || column > self.width-1 {
            return false;
        }

        // If cell exists, see if it's on fire
        let index = self.get_index(row, column);
        if self.cells[index] == Cell::Fire {
            return true;
        }
        return false;
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)], cell: Cell) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = cell;
        }
    }
}


/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        self.nexts = self.cells.clone();

        for row in (0..self.height).rev() {
            for col in (0..self.width).rev() {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbours = self.live_neighbour_count(row, col);

                let next_cell = match cell {
                    Cell::Dead  => self.life_state_next_tick(row, col, cell),
                    Cell::Alive => self.life_state_next_tick(row, col, cell),
                    Cell::Wood  => self.wood_state_next_tick(row, col),
                    Cell::Fire  => self.fire_state_next_tick(row, col),
                    Cell::Sand  => self.sand_state_next_tick(row, col),
                    _ => Cell::Dead
                };

                self.nexts[idx] = next_cell;
            }
        }
        self.cells = self.nexts.clone();
    }

    pub fn life_demo() -> Universe {
        utils::set_panic_hook();

        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if (i % 2 == 0 || i % 7 == 0) {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        let nexts = (0..width * height)
            .map(|i| { Cell::Dead }).collect();

        Universe {
            width,
            height,
            cells,
            nexts
        }
    }

    pub fn fire_demo() -> Universe {
        utils::set_panic_hook();

        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if (i == 0) {
                    Cell::Fire
                } else {
                    Cell::Wood
                }
            })
            .collect();

        let nexts = (0..width * height)
            .map(|i| { Cell::Dead }).collect();

        Universe {
            width,
            height,
            cells,
            nexts
        }
    }

    pub fn sand_demo() -> Universe {
        utils::set_panic_hook();

        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if (i % 3 == 0) {
                    Cell::Sand
                } else {
                    Cell::Dead
                }
            })
            .collect();

        let nexts = (0..width * height)
            .map(|i| { Cell::Dead }).collect();

        Universe {
            width,
            height,
            cells,
            nexts
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }


}
