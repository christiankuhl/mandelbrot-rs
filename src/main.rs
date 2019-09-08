use std::time::{Duration, Instant};
use std::thread::sleep;
use minifb::{Window, WindowOptions, MouseMode, MouseButton, Key};
use num::Complex;

const MAX_ITERATIONS: u32 = 255;
const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const START_RANGE: PlotRange = PlotRange { top_left: Complex {re: -2.0, im: 1.25},
                                           bottom_right: Complex {re: 1.0, im: -1.25}};
const ZOOM: f32 = 2.0;
const FRAME_DURATION: Duration = Duration::from_millis(17);

fn main() {
    let mut window = match Window::new("mandelbrot-rs", WIDTH, HEIGHT, WindowOptions::default()) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to create window {}", err);
            return;
        }
    };
    let mut app: Application = Application::new(&mut window);
    app.main_loop();
}

struct PlotRange {
    top_left: Complex<f32>,
    bottom_right: Complex<f32>
}

struct ApplicationSettings {
    zoom: f32,
    max_iterations: u32
}

impl PlotRange {
    pub fn index_to_point(&self, index: usize) -> Complex<f32> {
        Complex {re: ((index % WIDTH) as f32) / (WIDTH as f32)
                        * self.width() + self.top_left.re,
                 im: (((index / WIDTH) as f32).floor()) / (HEIGHT as f32)
                         * self.height() + self.top_left.im}
    }
    pub fn zoom_in(&mut self, point: &(f32, f32), settings: &ApplicationSettings) {
        let h = self.height();
        let w = self.width();
        let mid_x = point.0 / (WIDTH as f32) * w + self.top_left.re;
        let mid_y = point.1 / (HEIGHT as f32) * h + self.top_left.im;
        self.top_left = Complex {re: mid_x - w / (2.0 * settings.zoom),
                                 im: mid_y - h / (2.0 * settings.zoom)};
        self.bottom_right = Complex {re: mid_x + w / (2.0 * settings.zoom),
                                     im: mid_y + h / (2.0 * settings.zoom)};
    }
    pub fn height(&self) -> f32 {
        self.bottom_right.im - self.top_left.im
    }
    pub fn width(&self) -> f32 {
        self.bottom_right.re - self.top_left.re
    }
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

struct Application<'a> {
   plot_range: PlotRange,
   window: &'a mut Window,
   settings: ApplicationSettings,
   buffer: Vec<u32>
}

impl<'a> Application<'a> {
   pub fn new(window: &'a mut Window) -> Application<'a> {
       let buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
       let settings = ApplicationSettings {zoom: ZOOM, max_iterations: MAX_ITERATIONS};
       Application {plot_range: START_RANGE,
                    window: window,
                    settings: settings,
                    buffer: buffer}
   }
   fn update(&mut self) {
       for (index, value) in self.buffer.iter_mut().enumerate() {
           let z = self.plot_range.index_to_point(index);
           *value = 256 * escape_time(&z, &self.settings).unwrap_or(0.0) as u32;
       }
       self.window.update_with_buffer(&self.buffer).unwrap();
   }
   fn zoom_in(&mut self, point: (f32, f32)) {
        self.plot_range.zoom_in(&point, &self.settings);
        self.update();
   }
   fn main_loop(&mut self) {
       self.update();
       let mut start = Instant::now();
       while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
           let now = Instant::now();
           if let Some(wait_time) = FRAME_DURATION.checked_sub(now.duration_since(start)) {
               sleep(wait_time);
           }
           let left_down = self.window.get_mouse_down(MouseButton::Left);
           if left_down {
               if let Some(point) = self.window.get_mouse_pos(MouseMode::Clamp) {
                   self.zoom_in(point);
               }
           }
           self.window.update_with_buffer(&self.buffer).unwrap();
           start = now;
       }
   }
}