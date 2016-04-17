//! This program draws a growing, branching shape. The effect is something like
//! a colony of bacteria or the veins of a leaf.
//!
//! The algorithm is called DLA (diffusion limited aggregation) and it works
//! like this:
//!
//! *   Start with a single green seed pixel on a field of white.
//! *   Loop forever:
//!     *   Pick a point (x, y). Imagine some bit of food starting at that location.
//!     *   Inner loop:
//!         *   If (x, y) is adjacent to a green cell, the plant catches and eats
//!             the food.  So it grows in the direction of the food it just ate.
//!             Color the pixel (x, y) green and break out of the inner loop.
//!         *   Randomly perturb x and y (the food drifts to a neighboring pixel).
//!             If the new point (x, y) is off-screen, the food got away: break out
//!             of the inner loop.
//!
//! This program is homework for "Simulation of Biology", a remarkable course
//! by Greg Turk of Georgia Tech: <http://www.cc.gatech.edu/~turk/bio_sim/>.

// One obvious way to optimize this would be to store (in the grid) which cells
// are adjacent to green cells, rather than calling is_adjacent every time we
// want to know.

extern crate graphics;
extern crate piston;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use rand::Rng;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use glutin_window::GlutinWindow as Window;

pub const WINDOW_HEIGHT: u32 = 960;
pub const WINDOW_WIDTH: u32 = 1280;

pub const BLOCK_SIZE: u32 = 2;  // NOTE: WINDOW_HEIGHT and WINDOW_WIDTH must be divisible by this

pub const GRID_WIDTH: usize = (WINDOW_WIDTH / BLOCK_SIZE) as usize;
pub const GRID_HEIGHT: usize = (WINDOW_HEIGHT / BLOCK_SIZE) as usize;

pub const FRAME_DURATION: f64 = 0.1; // seconds

struct Grid<R: Rng> {
    cells: Vec<bool>,
    t: f64,
    rng: R,
    stickiness: f64,
    running: bool
}

impl<R: Rng> Grid<R> {
    fn new_empty(rng: R, stickiness: f64) -> Grid<R> {
        Grid {
            cells: vec![false; GRID_WIDTH * GRID_HEIGHT],
            t: 0.0,
            rng: rng,
            stickiness: stickiness,
            running: true
        }
    }

    fn new(rng: R, stickiness: f64) -> Grid<R> {
        let mut grid = Grid::new_empty(rng, stickiness);
        grid.set(GRID_WIDTH as i32 / 2, GRID_HEIGHT as i32 / 2);
        grid
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        0 < x &&
        x < GRID_WIDTH as i32 &&
        0 < y &&
        y < GRID_HEIGHT as i32
    }
    
    fn test(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.cells[y as usize * GRID_WIDTH + x as usize]
    }

    /// True if the given cell (x, y) is adjacent to any occupied cell.
    fn is_adjacent(&self, x: i32, y: i32) -> bool {
           self.test(x - 1, y - 1)
        || self.test(x    , y - 1)
        || self.test(x + 1, y - 1)
        || self.test(x - 1, y    )
        || self.test(x + 1, y    )
        || self.test(x - 1, y + 1)
        || self.test(x    , y + 1)
        || self.test(x + 1, y + 1)
    }

    fn set(&mut self, x: i32, y: i32) {
        self.cells[y as usize * GRID_WIDTH + x as usize] = true;
    }

    fn update(&mut self, args: &UpdateArgs) {
	self.t += args.dt;
	while self.t > FRAME_DURATION {
	    self.update_one_frame();
	    self.t -= FRAME_DURATION;
	}
    }

    fn update_one_frame(&mut self) {
        const DIRS: [(i32, i32); 8] = [
            (-1, -1), ( 0, -1), ( 1, -1),
            (-1,  0),           ( 1,  0),
            (-1,  1), ( 0,  1), ( 1,  1)];

        for _ in 0 .. 60 {
            let mut x = self.rng.gen_range(0, GRID_WIDTH as i32);
            let mut y = self.rng.gen_range(0, GRID_HEIGHT as i32);
            loop {
                if self.is_adjacent(x, y) && self.rng.gen::<f64>() < self.stickiness {
                    self.set(x, y);
                    break;
                }
                let &(dx, dy) = self.rng.choose(&DIRS).unwrap();
                x += dx;
                y += dy;
                if !self.in_bounds(x, y) {
                    break;
                }
            }
        }
    }
}

fn render<R: Rng>(grid: &Grid<R>, gl: &mut GlGraphics, args: &RenderArgs) {
    const WHITE:  [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const GREEN: [f32; 4] = [0.0, 0.4, 0.0, 1.0];

    gl.draw(args.viewport(), |c, gl| {
	graphics::clear(WHITE, gl);
	let tr = c.transform.scale(BLOCK_SIZE as f64, BLOCK_SIZE as f64);

        for y in 0 .. GRID_HEIGHT as i32 {
            for x in 0 .. GRID_WIDTH as i32 {
                if grid.test(x, y) {
                    let coords = [x as f64, y as f64, 1.0, 1.0];
                    rectangle(GREEN, coords, tr, gl);
                }
            }
        }
    });
}

fn odd_grid<R: Rng>(rng: R) -> Grid<R> {
    let mut grid = Grid::new_empty(rng, 0.1);
    let pi = ::std::f64::consts::PI;
    let cy = GRID_HEIGHT as f64 / 2.0;
    let cx = GRID_WIDTH as f64 / 2.0;
    let r = f64::min(cx, cy) * 0.9;

    const N: i32 = 100;
    for i in 0 .. N {
        let t = i as f64 / N as f64;
        let a = 3.0 * pi * t;
        let x = cx + t * r * a.cos();
        let y = cy + t * r * a.sin();
        grid.set(x as i32, y as i32);
    }

    grid
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    assert!(WINDOW_WIDTH % BLOCK_SIZE == 0);
    assert!(WINDOW_HEIGHT % BLOCK_SIZE == 0);
    let window: Window = WindowSettings::new(
	"DLA".to_string(),
	[WINDOW_WIDTH, WINDOW_HEIGHT])
	.opengl(opengl)
	.exit_on_esc(true)
	.build()
	.unwrap();

    let mut gl = GlGraphics::new(opengl);

    // Fast XorShift random number generator, seeded from a better (but slower)
    // source of randomness.
    let rng: rand::XorShiftRng = rand::random();
    let mut grid = Grid::new(rng, 0.1);

    for e in window.events() {
	match e {
	    Event::Render(ref r) =>
                render(&grid, &mut gl, r),
	    Event::Update(ref u) =>
		if grid.running {
		    grid.update(u);
		},
	    Event::Input(Input::Press(Button::Keyboard(Key::Space))) =>
                grid.running = !grid.running,
	    Event::Input(Input::Press(Button::Keyboard(Key::D1))) =>
                grid = Grid::new(grid.rng, 1.0),
	    Event::Input(Input::Press(Button::Keyboard(Key::D2))) =>
                grid = Grid::new(grid.rng, 0.1),
	    Event::Input(Input::Press(Button::Keyboard(Key::D3))) =>
                grid = Grid::new(grid.rng, 0.01),
	    Event::Input(Input::Press(Button::Keyboard(Key::D0))) =>
                grid = odd_grid(grid.rng),
	    _ => {}
	}
    }
}
