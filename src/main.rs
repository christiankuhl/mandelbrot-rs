use std::time::{Duration, Instant};
use std::thread::sleep;
use std::iter::Cycle;
use std::slice::Iter;
use minifb::{Window, WindowOptions, MouseMode, MouseButton, Key};
use num::Complex;

const MAX_ITERATIONS: u32 = 255;
const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const START_RANGE: PlotRange = PlotRange { top_left: Complex {re: -2.0, im: 1.25},
                                           bottom_right: Complex {re: 1.0, im: -1.25}};
const ZOOM: f32 = 2.0;
const FRAME_DURATION: Duration = Duration::from_millis(17);
const ACTIVE_KEYS: [Key; 7] = [Key::Left, Key::Right, Key::Up, Key::Down, Key::Q, Key::Escape, Key::C];
const STEP_SIZE: f32 = 0.05;

fn main() {
    let mut window = Window::new("mandelbrot-rs", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut app: Application = Application::new(&mut window);
    app.main_loop();
}

fn escape_time(c: &Complex<f32>, settings: &ApplicationSettings) -> Option<f32> {
    let mut z = Complex {re: 0.0, im: 0.0};
    for i in 0..settings.max_iterations {
        z = z*z + c;
        if z.norm_sqr() > 4.0 {
            let shade = 1.0 - (z.norm_sqr().log2() / 2.0).ln();
            return Some((i as f32) + shade)
        }
    }
    None
}

struct ApplicationSettings {
    zoom: f32,
    max_iterations: u32,
    colour: u32
}

struct Application<'a> {
   plot_range: PlotRange,
   window: &'a mut Window,
   settings: ApplicationSettings,
   buffer: Vec<u32>,
   colours: Cycle<Iter<'a, u32>>
}

impl<'a> Application<'a> {
   pub fn new(window: &'a mut Window) -> Application<'a> {
       let buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
       let settings = ApplicationSettings {zoom: ZOOM, max_iterations: MAX_ITERATIONS, colour: 256};
       let colour_iterator = [65536, 1, 256].iter().cycle();
       Application {plot_range: START_RANGE,
                    window: window,
                    settings: settings,
                    buffer: buffer,
                    colours: colour_iterator}
   }
   fn update(&mut self) {
       for (index, value) in self.buffer.iter_mut().enumerate() {
           let z = self.plot_range.index_to_point(index);
           *value = self.settings.colour * escape_time(&z, &self.settings).unwrap_or(0.0) as u32;
       }
       self.window.update_with_buffer(&self.buffer).unwrap();
   }
   fn zoom(&mut self, point: &(f32, f32), out: bool) {
        self.plot_range.zoom(&point, out, &self.settings);
        self.update();
   }
   fn shift(&mut self, direction: Key){
       self.plot_range.shift(direction);
       self.update();
   }
   fn toggle_colour(&mut self) {
       self.settings.colour = *self.colours.next().unwrap();
       self.update();
   }
   fn main_loop(&mut self) {
       self.update();
       let mut start = Instant::now();
       while self.window.is_open() {
           let now = Instant::now();
           if let Some(wait_time) = FRAME_DURATION.checked_sub(now.duration_since(start)) {
               sleep(wait_time);
           }
           let left_button = self.window.get_mouse_down(MouseButton::Left);
           let right_button = self.window.get_mouse_down(MouseButton::Right);
           if left_button || right_button {
               if let Some(point) = self.window.get_mouse_pos(MouseMode::Clamp) {
                   self.zoom(&point, right_button);
               }
           }
           match self.key_press() {
               Some(Key::Left) => self.shift(Key::Left),
               Some(Key::Right) => self.shift(Key::Right),
               Some(Key::Up) => self.shift(Key::Up),
               Some(Key::Down) => self.shift(Key::Down),
               Some(Key::Q) => return,
               Some(Key::Escape) => return,
               Some(Key::C) => self.toggle_colour(),
               _ => ()
           }
           self.window.update_with_buffer(&self.buffer).unwrap();
           start = now;
       }
   }
   fn key_press(&self) -> Option<Key> {
       match ACTIVE_KEYS.iter().find(|&key| self.window.is_key_down(*key)) {
           Some(key) => Some(*key),
           None => None
       }
   }
}

struct PlotRange {
    top_left: Complex<f32>,
    bottom_right: Complex<f32>
}

impl PlotRange {
    pub fn index_to_point(&self, index: usize) -> Complex<f32> {
        Complex {re: ((index % WIDTH) as f32) / (WIDTH as f32)
                        * self.width() + self.top_left.re,
                 im: (((index / WIDTH) as f32).floor()) / (HEIGHT as f32)
                         * self.height() + self.top_left.im}
    }
    pub fn zoom(&mut self, point: &(f32, f32), out: bool, settings: &ApplicationSettings) {
        let h = self.height();
        let w = self.width();
        let mut z = settings.zoom;
        if out {
            z = 1.0 / z;
        }
        let mid_x = point.0 / (WIDTH as f32) * w + self.top_left.re;
        let mid_y = point.1 / (HEIGHT as f32) * h + self.top_left.im;
        self.top_left = Complex {re: mid_x - w / (2.0 * z),
                                 im: mid_y - h / (2.0 * z)};
        self.bottom_right = Complex {re: mid_x + w / (2.0 * z),
                                     im: mid_y + h / (2.0 * z)};
    }
    pub fn shift(&mut self, direction: Key) {
        let w = self.width() * STEP_SIZE;
        let delta = match direction {
            Key::Left => Complex {re: -w, im: 0.0},
            Key::Right => Complex {re: w, im: 0.0},
            Key::Up => Complex {re: 0.0, im: w},
            Key::Down => Complex {re: 0.0, im: -w},
            _ => Complex {re: 0.0, im: 0.0}
        };
        self.top_left += delta;
        self.bottom_right += delta;
    }
    pub fn height(&self) -> f32 {
        self.bottom_right.im - self.top_left.im
    }
    pub fn width(&self) -> f32 {
        self.bottom_right.re - self.top_left.re
    }
}
