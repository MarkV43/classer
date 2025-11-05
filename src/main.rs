mod discriminate;

use std::{env, path, time::Duration};

use ggez::{
    glam::Vec2,
    graphics::{Canvas, Color, DrawMode, DrawParam, FillOptions, Image, InstanceArray, Rect},
    mint::Point2,
    winit::event::MouseButton,
    *,
};
use strum::{EnumIter, IntoEnumIterator};

use crate::discriminate::{LinearDiscrimination, Point, linear_discriminate};

const MARGIN: f32 = 5.0;
const BTN_SIZE: f32 = 60.0;
const BTN_SPACING: f32 = 70.0;

struct Button {
    rect: Rect,
    text: String,
    image: Image,
}

impl Button {
    fn update(
        &mut self,
        ctx: &mut Context,
        callback: impl FnOnce() -> (),
    ) -> Result<(), GameError> {
        if ctx
            .mouse
            .button_just_pressed(winit::event::MouseButton::Left)
        {
            let pos = ctx.mouse.position();

            if self.rect.contains(pos) {
                (callback)();
            }
        }

        Ok(())
    }

    fn draw(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        highlight: bool,
    ) -> Result<(), GameError> {
        let mesh = ggez::graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            self.rect,
            if highlight {
                Color::from_rgb(200, 200, 200)
            } else {
                Color::WHITE
            },
        )?;

        canvas.draw(&mesh, DrawParam::new());

        let text = ggez::graphics::Text::new(&self.text);

        canvas.draw(
            &text,
            Vec2::new(self.rect.x + 10.0, self.rect.center().y - 10.0),
        );

        let center = self.rect.center();
        let center = Vec2::new(center.x, center.y);
        let img_size = Vec2::new(self.image.width() as f32, self.image.height() as f32);

        let scale = glam::Vec2::new(0.1, 0.1);
        let dst = center - 0.5 * scale * img_size;

        canvas.draw(
            &self.image,
            graphics::DrawParam::new().dest(dst).scale(scale),
        );

        Ok(())
    }
}

#[derive(EnumIter, PartialEq)]
enum InputMode {
    Add,
    Remove,
    Paint,
}

enum DiscriminationKind {
    Linear,
    Quadratic,
    Polynomial,
}

fn find_line_endpoints(vec_a: [f32; 2], scl_b: f32, w: f32, h: f32, eps: f32) -> [Vec2; 2] {
    const M: f32 = 2.0 * MARGIN + BTN_SIZE;

    let a1 = vec_a[0];
    let a2 = vec_a[1];

    if a1.abs() <= eps && a2.abs() <= eps {
        panic!("vec_a cannot be (0,0)");
    }

    let mut candidates: Vec<Vec2> = Vec::with_capacity(4);

    // left edge x = 0  -> y = -b / a2
    if a2.abs() > eps {
        let y = scl_b / a2;
        if y >= M - eps && y <= h + eps {
            candidates.push(Vec2::new(0.0, y.clamp(M, h)));
        }
    }

    // right edge x = W -> y = -(a1*W + b) / a2
    if a2.abs() > eps {
        let y = -(a1 * w - scl_b) / a2;
        if y >= M - eps && y <= h + eps {
            candidates.push(Vec2::new(w, y.clamp(M, h)));
        }
    }

    // bottom edge y = M -> x = -(a2*M + b) / a1
    if a1.abs() > eps {
        let x = -(a2 * M - scl_b) / a1;
        if x >= -eps && x <= w + eps {
            candidates.push(Vec2::new(x.clamp(0.0, w), M));
        }
    }

    // top edge y = H -> x = -(a2*H + b) / a1
    if a1.abs() > eps {
        let x = -(a2 * h - scl_b) / a1;
        if x >= -eps && x <= w + eps {
            candidates.push(Vec2::new(x.clamp(0.0, w), h));
        }
    }

    // remove duplicates within tolerance
    let mut unique: Vec<Vec2> = Vec::with_capacity(2);
    for p in candidates {
        let mut is_dup = false;
        for q in &unique {
            if (p.x - q.x).abs() <= 1e-6 && (p.y - q.y).abs() <= 1e-6 {
                is_dup = true;
                break;
            }
        }
        if !is_dup {
            unique.push(p);
            if unique.len() == 2 {
                break;
            }
        }
    }

    // If nothing found, handle exact vertical or horizontal line cases
    if unique.is_empty() {
        // vertical line: a2 == 0 -> x = -b / a1
        if a2.abs() <= eps && a1.abs() > eps {
            let x = scl_b / a1;
            if x >= 0.0 - eps && x <= w + eps {
                return [Vec2::new(x.clamp(0.0, w), M), Vec2::new(x.clamp(0.0, w), h)];
            }
        }
        // horizontal line: a1 == 0 -> y = -b / a2
        if a1.abs() <= eps && a2.abs() > eps {
            let y = scl_b / a2;
            if y >= M - eps && y <= h + eps {
                return [Vec2::new(0.0, y.clamp(M, h)), Vec2::new(w, y.clamp(M, h))];
            }
        }

        // no intersection
        println!("vec_a: {vec_a:?}\nscl_b: {scl_b}");
        println!("bounds: 0..{w} ; {M}..{h}");
        panic!("No intersection found");
    }

    if unique.len() == 1 {
        return [unique[0], unique[0]];
    }

    // two distinct points
    [unique[0], unique[1]]
}

struct State {
    dt: Duration,
    buttons_mode: Vec<Button>,
    buttons_right: Vec<Button>,
    input_mode: InputMode,
    input_radius: f64,
    discr_kind: DiscriminationKind,
    solution: Option<LinearDiscrimination>,

    white_points: Vec<Point>,
    black_points: Vec<Point>,
}

impl State {
    pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
        Ok(Self {
            dt: std::time::Duration::new(0, 0),
            buttons_mode: vec![
                Button {
                    rect: Rect::new(5.0 + 0.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                    text: "+".to_owned(),
                    image: Image::from_path(ctx, "/plus.png")?,
                },
                Button {
                    rect: Rect::new(5.0 + 1.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                    text: "-".to_owned(),
                    image: Image::from_path(ctx, "/minus.png")?,
                },
                Button {
                    rect: Rect::new(5.0 + 2.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                    text: "Paint".to_owned(),
                    image: Image::from_path(ctx, "/fill.png")?,
                },
            ],
            buttons_right: vec![
                Button {
                    rect: Rect::new(5.0 + 0.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                    text: "dir1".to_owned(),
                    image: Image::from_path(ctx, "/trash.png")?,
                },
                Button {
                    rect: Rect::new(5.0 + 1.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                    text: "dir2".to_owned(),
                    image: Image::from_path(ctx, "/trash.png")?,
                },
            ],
            input_mode: InputMode::Add,
            input_radius: 50.0,
            discr_kind: DiscriminationKind::Linear,
            solution: None,

            white_points: Vec::new(),
            black_points: Vec::new(),
        })
    }
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let (w, _) = ctx.gfx.drawable_size();

        self.dt = ctx.time.delta();

        for (btn, mode) in self.buttons_mode.iter_mut().zip(InputMode::iter()) {
            btn.update(ctx, || self.input_mode = mode)?;
        }
        for (i, btn) in self.buttons_right.iter_mut().enumerate() {
            btn.rect.x = w - (i + 1) as f32 * (btn.rect.w + 10.0);
        }
        self.buttons_right[0].update(ctx, || {
            self.white_points.clear();
            self.black_points.clear();
        })?;
        self.buttons_right[1].update(ctx, || println!("Change solution type"))?;

        // Handle clicks
        let mouse_pos = ctx.mouse.position();

        // println!("mpos: {mouse_pos:?}");

        if mouse_pos.y <= BTN_SIZE + 2.0 * MARGIN {
            return Ok(());
        }

        let mut changed = false;

        match self.input_mode {
            InputMode::Add => {
                if ctx.mouse.button_just_pressed(MouseButton::Left) {
                    self.black_points.push([mouse_pos.x, mouse_pos.y]);
                    changed = true;
                } else if ctx.mouse.button_just_pressed(MouseButton::Right) {
                    self.white_points.push([mouse_pos.x, mouse_pos.y]);
                    changed = true;
                }
            }
            InputMode::Remove => todo!(),
            InputMode::Paint => todo!(),
        }

        if !changed {
            return Ok(());
        }

        if self.black_points.is_empty() || self.white_points.is_empty() {
            self.solution = None;
            return Ok(());
        }

        println!("{:?}\n{:?}", self.black_points, self.white_points);

        // Run discrimination algorithm
        match self.discr_kind {
            DiscriminationKind::Linear => {
                self.solution = linear_discriminate(&self.black_points, &self.white_points)
                    .ok()
                    .inspect(|x| println!("{:?}", x.vec_a));
            }
            DiscriminationKind::Quadratic => todo!(),
            DiscriminationKind::Polynomial => todo!(),
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        let (w, h) = ctx.gfx.drawable_size();

        // Draw the discrimination line if there is one
        if let Some(sol) = &self.solution {
            let points = find_line_endpoints(sol.vec_a, sol.scl_b, w, h, 1e-12);
            let line = ggez::graphics::Mesh::new_line(ctx, &points, 1.5, Color::GREEN)?;
            canvas.draw(&line, DrawParam::new());
        }

        // Draw the circles as an instanced mesh
        let circle = ggez::graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            3.0,
            0.1,
            Color::WHITE,
        )?;

        let mut instance_array = InstanceArray::new(ctx, None);

        for pnt in &self.black_points {
            instance_array.push(
                DrawParam::new()
                    .dest(Point2::from_slice(pnt))
                    .color(Color::RED),
            );
        }
        for pnt in &self.white_points {
            instance_array.push(
                DrawParam::new()
                    .dest(Point2::from_slice(pnt))
                    .color(Color::BLUE),
            );
        }

        canvas.draw_instanced_mesh(circle, &instance_array, DrawParam::new());

        // Draw UI on top
        let rect = ggez::graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0.0, 0.0, w, BTN_SIZE + 2.0 * MARGIN),
            Color::from_rgb(100, 100, 100),
        )?;

        canvas.draw(&rect, DrawParam::new());

        let fps = 1.0 / self.dt.as_secs_f64();

        let mut text = ggez::graphics::Text::new(format!("{fps:.0}"));
        text.set_scale(50.0);
        let txt_size = text.measure(ctx)?;
        canvas.draw(
            &text,
            Vec2::new(
                3.0 * (BTN_SIZE + MARGIN) + MARGIN + txt_size.x / 2.0,
                MARGIN,
            ),
        );

        for (btn, mode) in self.buttons_mode.iter_mut().zip(InputMode::iter()) {
            btn.draw(ctx, &mut canvas, self.input_mode == mode)?;
        }
        for btn in &mut self.buttons_right {
            btn.draw(ctx, &mut canvas, false)?;
        }

        // Copy canvas to the screen
        canvas.finish(ctx)?;

        Ok(())
    }
}

fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let mut c = conf::Conf::new();
    c.window_setup = c.window_setup.vsync(false);
    let (mut ctx, event_loop) = ContextBuilder::new("hello_ggez", "test")
        .add_resource_path(resource_dir)
        .default_conf(c)
        .build()
        .unwrap();

    let state = State::new(&mut ctx).unwrap();

    event::run(ctx, event_loop, state).unwrap()
}
