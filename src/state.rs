use ggez::{
    Context, GameError, event,
    glam::Vec2,
    graphics::{Canvas, Color, DrawMode, DrawParam, Image, InstanceArray, Rect},
    winit::{
        event::MouseButton,
        keyboard::{KeyCode, PhysicalKey},
    },
};
use std::time::Duration;
use strum::{EnumIter, IntoEnumIterator};

use crate::constants::*;
use crate::discriminate::{LinearDiscrimination, Point, linear_discriminate};
use crate::ui::Button;
use crate::utils::{paint_by_radius, remove_by_radius};

#[derive(EnumIter, PartialEq, Clone, Copy)]
pub enum InputMode {
    Add,
    Remove,
    Paint,
}

#[derive(Clone, Copy)]
pub enum DiscriminationKind {
    Linear,
    Quadratic,
    Polynomial,
}

pub struct State {
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
                image: Image::from_path(ctx, "/del.png")?,
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

    pub fn mouse_wheel_event(
        &mut self,
        _ctx: &mut Context,
        _x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        self.input_radius += self.input_radius * y * 0.1;
        Ok(())
    }
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let (w, h) = ctx.gfx.drawable_size();

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
                let discr = linear_discriminate(&self.black_points, &self.white_points).ok();
                self.solution = discr;
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

        for pos in &self.black_points {
            instance_array.push(
                DrawParam::new()
                    .dest(Vec2::from_slice(pos))
                    .color(Color::WHITE),
            );
        }
        for pos in &self.white_points {
            instance_array.push(
                DrawParam::new()
                    .dest(Vec2::from_slice(pos))
                    .color(Color::BLACK),
            );
        }

        if ctx
            .keyboard
            .is_physical_key_just_pressed(&PhysicalKey::Code(KeyCode::KeyH))
        {
            println!("let xs = {:.16?};", self.black_points);
            println!("let ys = {:.16?};", self.white_points);
            println!("discr: {:?}", self.solution);

            let sol = linear_discriminate(&self.black_points, &self.white_points);
            println!("sol: {sol:?}");
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

    // Explicitly implemented to forward to our struct method
    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        self.mouse_wheel_event(ctx, x, y)
    }
}
