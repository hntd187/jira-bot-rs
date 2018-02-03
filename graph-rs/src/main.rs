extern crate image;

#[macro_use]
extern crate conrod;
extern crate find_folder;

use conrod::backend::glium::glium;
use conrod::backend::glium::glium::glutin::*;
use conrod::backend::glium::glium::*;
use conrod::backend::glium::glium::texture::Texture2d;
use conrod::image::Map;

use conrod::{widget, Positionable, Sizeable, Widget, Borderable, Colorable};
use conrod::backend::glium::Renderer;
use conrod::backend::winit::convert_event;
use conrod::position::{Padding, Position, Relative, Align, Direction};

pub fn theme() -> conrod::Theme {

  conrod::Theme {
    name: "Demo Theme".to_string(),
    padding: Padding::none(),
    x_position: Position::Relative(Relative::Align(Align::Start), None),
    y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
    background_color: conrod::color::LIGHT_CHARCOAL,
    shape_color: conrod::color::BLACK,
    border_color: conrod::color::BLACK,
    border_width: 0.0,
    label_color: conrod::color::WHITE,
    font_id: None,
    font_size_large: 26,
    font_size_medium: 18,
    font_size_small: 12,
    widget_styling: conrod::theme::StyleMap::default(),
    mouse_drag_threshold: 0.0,
    double_click_threshold: std::time::Duration::from_millis(500),
  }
}

widget_ids! {
    pub struct Ids {
        canvas,
        title,
        plot_path,
        canvas_scrollbar,
        mouse_pos,
        x_axis,
        y_axis,
        top,
        right,
        bottom,
        left
    }
}

pub struct EventLoop {
  ui_needs_update: bool,
  last_update: std::time::Instant,
}

#[derive(Default)]
pub struct PlotState(f64, f64);

impl PlotState {
  pub fn new() -> Self {
    PlotState(0.0, 0.0)
  }
}

impl EventLoop {
  pub fn new() -> Self {
    EventLoop {
      last_update: std::time::Instant::now(),
      ui_needs_update: true,
    }
  }

  pub fn next(&mut self, events_loop: &mut glium::glutin::EventsLoop) -> Vec<glium::glutin::Event> {
    let last_update = self.last_update;
    let sixteen_ms = std::time::Duration::from_millis(16);
    let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
    if duration_since_last_update < sixteen_ms {
      std::thread::sleep(sixteen_ms - duration_since_last_update);
    }

    let mut events = Vec::new();
    events_loop.poll_events(|event| events.push(event));

    if events.is_empty() && !self.ui_needs_update {
      events_loop.run_forever(|event| {
        events.push(event);
        glium::glutin::ControlFlow::Break
      });
    }

    self.ui_needs_update = false;
    self.last_update = std::time::Instant::now();

    events
  }

  pub fn needs_update(&mut self) {
    self.ui_needs_update = true;
  }
}

fn normal(x: f32, mean: f32, stddev: f32) -> f32 {
  use std::f32::consts::{PI, E};
  let exp = -(x - mean).powi(2) / (2.0 * stddev.powi(2));
  (1.0 / stddev.powi(2) * (2.0 * PI).sqrt()) * E.powf(exp)
}

fn gui(ui: &mut conrod::UiCell, ids: &Ids, state: &PlotState) {
  let canvas = widget::Canvas::new();
  canvas.pad(10.0)
    .border(1.5)
    .set(ids.canvas, ui);

  let mos_pos = format!("({:03}, {:03})", state.0, state.1);
  widget::Text::new(&mos_pos)
    .font_size(12)
    .bottom_left_of(ids.canvas)
    .set(ids.mouse_pos, ui);

  let min_x = -5.0;
  let max_x = 5.0;
  let min_y = 0.0;
  let max_y = 1.0;

  let x_axis_start = [-275.0, 275.0];
  let x_axis_end = [-275.0, -275.0];


  let y_axis_start = [-275.0, -275.0];
  let y_axis_end = [275.0, -275.0];

  let top_start = [275.0, 275.0];
  let top_end = [-275.0, 275.0];

  let right_start = [275.0, -275.0];
  let right_end = [275.0, 275.0];

  widget::Line::new(y_axis_start, y_axis_end).thickness(1.0).mid_left_of(ids.canvas).set(ids.bottom, ui);
  widget::Line::new(top_start, top_end).thickness(1.0).mid_left_of(ids.canvas).color(theme().background_color).set(ids.top, ui);

  widget::Line::new(x_axis_start, x_axis_end).thickness(1.0).mid_bottom_of(ids.canvas).set(ids.left, ui);
  widget::Line::new(right_start, right_end).thickness(1.0).mid_bottom_of(ids.canvas).color(theme().background_color).set(ids.right, ui);

  let mut plot = widget::PlotPath::new(min_x, max_x, min_y, max_y, |x: f32| { normal(x, 0.0, 1.0) });

  plot = plot.kid_area_wh_of(ids.canvas)
    //.x_position(Position::Absolute(0.0))
    //.y_position(Position::Absolute(-200.0))
    .right_from(ids.left, 0.0)
    .up_from(ids.bottom, 0.0)
    .left_from(ids.right, 0.0)
    .down_from(ids.top, 0.0)
    .w(550.0)
    .h(350.0);
    //.set(ids.plot_path, ui);

  let dem = plot.get_wh(ui.as_ref()).unwrap();
  let xpos = plot.get_x_position(ui.as_ref());
  let ypos = plot.get_y_position(ui.as_ref());


  plot.set(ids.plot_path, ui);
  println!("{:?}, {:?}", xpos, ypos);
  println!("{:?}", dem);
}

fn main() {
  let width = 640;
  let height = 640;

  let mut events_loop = EventsLoop::new();
  let window = WindowBuilder::new()
    .with_title("Sweet Window Brah")
    .with_dimensions(width, height);
  let context = ContextBuilder::new()
    .with_vsync(true)
    .with_multisampling(8);
  let display = Display::new(window, context, &events_loop).unwrap();

  let mut ui = conrod::UiBuilder::new([width as f64, height as f64])
    .theme(theme())
    .build();

  ui.fonts.insert_from_file("assets/NotoSans-Regular.ttf").unwrap();

  let image_map: Map<Texture2d> = Map::new();
  //let rust_logo = image_map.insert(load_rust_logo(&display));
  let ids = Ids::new(ui.widget_id_generator());
  let mut renderer = Renderer::new(&display).unwrap();
  let mut pos = PlotState::new();
  let mut event_loop = EventLoop::new();
  'main: loop {
    for event in event_loop.next(&mut events_loop) {
      if let Some(event) = convert_event(event.clone(), &display) {
        ui.handle_event(event);
        event_loop.needs_update();
      }
      match event {
        Event::WindowEvent { event, .. } => match event {
          WindowEvent::Closed | WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. }, .. } => break 'main,
          WindowEvent::CursorMoved { position, .. } => {
            pos.0 = position.0;
            pos.1 = position.1;
          }
          _ => (),
        },
        _ => (),
      }
    }

    gui(&mut ui.set_widgets(), &ids, &pos);

    if let Some(primitives) = ui.draw_if_changed() {
      renderer.fill(&display, primitives, &image_map);
      let mut target = display.draw();
      target.clear_color(0.0, 0.0, 0.0, 1.0);
      renderer.draw(&display, &mut target, &image_map).unwrap();
      target.finish().unwrap();
    }
  }
}

