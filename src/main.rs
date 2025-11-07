mod discriminate;

use std::{env, path, time::Duration, vec};

use ggez::{
    event::EventHandler,
    glam::Vec2,
    graphics::{
        Canvas, Color, DrawMode, DrawParam, Image, InstanceArray, MeshBuilder, Rect, ScreenImage,
        StrokeOptions,
    },
    mint::Point2,
    winit::event::{MouseButton, MouseScrollDelta},
    *,
};
use strum::{EnumIter, IntoEnumIterator};

use crate::discriminate::{LinearDiscrimination, Point, linear_discriminate};

const MARGIN: f32 = 5.0;
const BTN_SIZE: f32 = 60.0;
const BTN_SPACING: f32 = 70.0;

// Colors
const LIGHT_GRAY: Color = Color::new(199.0 / 255.0, 193.0 / 255.0, 174.0 / 255.0, 1.0);
const DARK_GRAY: Color = Color::new(40.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0, 0.4);
const GRAY: Color = Color::new(66.0 / 255.0, 66.0 / 255.0, 66.0 / 255.0, 1.0);

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
            if highlight { LIGHT_GRAY } else { DARK_GRAY },
        )?;

        canvas.draw(&mesh, DrawParam::new());

        // Draw the button's border
        let border = ggez::graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::stroke(2.0),
            self.rect,
            Color::BLACK,
        )?;
        canvas.draw(&border, DrawParam::new());

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

struct State {
    dt: Duration,
    buttons_mode: Vec<Button>,
    buttons_right: Vec<Button>,
    input_mode: InputMode,
    input_radius: f32,
    discr_kind: DiscriminationKind,
    solution: Option<LinearDiscrimination>,

    white_points: Vec<Point>,
    black_points: Vec<Point>,

    shader: ggez::graphics::Shader,
    shader_params: ggez::graphics::ShaderParams<LinearDiscrimination>,
}

impl State {
    pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
        let buttons_mode = vec![
            Button {
                rect: Rect::new(5.0 + 0.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "".to_owned(),
                image: Image::from_path(ctx, "/add.png")?,
            },
            Button {
                rect: Rect::new(5.0 + 1.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "".to_owned(),
                image: Image::from_path(ctx, "/backspace.png")?,
            },
            Button {
                rect: Rect::new(5.0 + 2.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "".to_owned(),
                image: Image::from_path(ctx, "/fill.png")?,
            },
        ];

        let buttons_right = vec![
            Button {
                rect: Rect::new(5.0 + 0.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "".to_owned(),
                image: Image::from_path(ctx, "/trash.png")?,
            },
            Button {
                rect: Rect::new(5.0 + 1.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "".to_owned(),
                image: Image::from_path(ctx, "/btn_linear.png")?,
            },
        ];

        let shader = ggez::graphics::ShaderBuilder::new()
            .fragment_path("/fragment.wgsl")
            .build(ctx)?;
        let shader_params =
            ggez::graphics::ShaderParamsBuilder::new(&LinearDiscrimination::none()).build(ctx);

        Ok(Self {
            dt: std::time::Duration::new(0, 0),
            buttons_mode,
            buttons_right,
            input_mode: InputMode::Add,
            input_radius: 50.0,
            discr_kind: DiscriminationKind::Linear,
            solution: None,

            white_points: Vec::new(),
            black_points: Vec::new(),

            shader,
            shader_params,
        })
    }
}

fn remove_by_radius(vec: &mut Vec<[f32; 2]>, center: Vec2, radius: f32, changed: &mut bool) {
    let radius_sq = radius * radius;
    let mut i = 0;
    loop {
        if i >= vec.len() {
            break;
        }
        let pos = Vec2::from_array(vec[i]);
        if pos.distance_squared(center) <= radius_sq {
            vec.swap_remove(i);
            *changed = true;
        } else {
            i += 1;
        }
    }
}

fn paint_by_radius(
    movefrom: &mut Vec<[f32; 2]>,
    moveto: &mut Vec<[f32; 2]>,
    center: Vec2,
    radius: f32,
    changed: &mut bool,
) {
    let radius_sq = radius * radius;
    let mut i = 0;
    loop {
        if i >= movefrom.len() {
            break;
        }
        let pos = Vec2::from_array(movefrom[i]);
        if pos.distance_squared(center) <= radius_sq {
            moveto.push(pos.into());
            movefrom.swap_remove(i);
            *changed = true;
        } else {
            i += 1;
        }
    }
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let (w, _) = ctx.gfx.drawable_size();

        let mut changed = false;

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
            changed = true;
        })?;
        self.buttons_right[1].update(ctx, || println!("Change solution type"))?;

        // Handle clicks
        let mouse_pos = ctx.mouse.position();

        // println!("mpos: {mouse_pos:?}");

        if mouse_pos.y > BTN_SIZE + 2.0 * MARGIN {
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
                InputMode::Remove => {
                    if ctx.mouse.button_pressed(MouseButton::Left) {
                        let mpos = mouse_pos.into();
                        remove_by_radius(
                            &mut self.black_points,
                            mpos,
                            self.input_radius,
                            &mut changed,
                        );
                        remove_by_radius(
                            &mut self.white_points,
                            mpos,
                            self.input_radius,
                            &mut changed,
                        );
                    }
                }
                InputMode::Paint => {
                    if ctx.mouse.button_pressed(MouseButton::Left) {
                        paint_by_radius(
                            &mut self.white_points,
                            &mut self.black_points,
                            mouse_pos.into(),
                            self.input_radius,
                            &mut changed,
                        );
                    } else if ctx.mouse.button_pressed(MouseButton::Right) {
                        paint_by_radius(
                            &mut self.black_points,
                            &mut self.white_points,
                            mouse_pos.into(),
                            self.input_radius,
                            &mut changed,
                        );
                    }
                }
            }
        }

        if !changed {
            return Ok(());
        }

        if self.black_points.is_empty() || self.white_points.is_empty() {
            self.solution = None;
            self.shader_params
                .set_uniforms(ctx, &LinearDiscrimination::none());
            return Ok(());
        }

        // Run discrimination algorithm
        match self.discr_kind {
            DiscriminationKind::Linear => {
                self.solution = linear_discriminate(&self.black_points, &self.white_points).ok();
            }
            DiscriminationKind::Quadratic => todo!(),
            DiscriminationKind::Polynomial => todo!(),
        }

        // Update shader params
        self.shader_params.set_uniforms(
            ctx,
            &self
                .solution
                .clone()
                .unwrap_or(LinearDiscrimination::none()),
        );

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, DARK_GRAY);
        let (w, h) = ctx.gfx.drawable_size();

        // Draw the discrimination background
        if self.solution.is_some() {
            canvas.set_shader(&self.shader);
            canvas.set_shader_params(&self.shader_params);
            let rect = ggez::graphics::Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect::new(0.0, 0.0, w, h),
                LIGHT_GRAY,
            )?;
            canvas.draw(&rect, Vec2::ZERO);
            canvas.set_default_shader();
        }

        // Draw the circle around the mouse
        match &self.input_mode {
            InputMode::Add => {}
            InputMode::Remove | InputMode::Paint => {
                let circle = ggez::graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::stroke(2.0),
                    ctx.mouse.position(),
                    self.input_radius,
                    0.1,
                    Color::RED,
                )?;
                canvas.draw(&circle, DrawParam::new());
            }
        }

        // Draw the circles as an instanced mesh
        let circle = ggez::graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Vec2::new(0.0, 0.0),
            5.0,
            1.0,
            Color::WHITE,
        )?;

        let mut instance_array = InstanceArray::new(ctx, None);

        for pnt in &self.black_points {
            instance_array.push(
                DrawParam::new()
                    .dest(Point2::from_slice(pnt))
                    .color(Color::WHITE),
            );
        }
        for pnt in &self.white_points {
            instance_array.push(
                DrawParam::new()
                    .dest(Point2::from_slice(pnt))
                    .color(Color::BLACK),
            );
        }

        canvas.draw_instanced_mesh(circle, &instance_array, DrawParam::new());

        // Draw UI on top
        let rect = ggez::graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0.0, 0.0, w, BTN_SIZE + 2.0 * MARGIN),
            GRAY,
        )?;

        canvas.draw(&rect, DrawParam::new());

        // Draw the header's border
        let border = ggez::graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::stroke(1.0),
            Rect::new(0.0, 0.0, w, BTN_SIZE + 2.0 * MARGIN),
            Color::BLACK,
        )?;
        canvas.draw(&border, DrawParam::new());

        /*
                // Exibir FPS no header
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
        */

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

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) -> Result<(), GameError> {
        self.input_radius += self.input_radius * y * 0.1;

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
    c.window_setup = c.window_setup.vsync(false).samples(conf::NumSamples::Four);
    let (mut ctx, event_loop) = ContextBuilder::new("hello_ggez", "test")
        .add_resource_path(resource_dir)
        .default_conf(c)
        .build()
        .unwrap();

    let state = State::new(&mut ctx).unwrap();

    event::run(ctx, event_loop, state).unwrap()
}
