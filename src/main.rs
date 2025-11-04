mod discriminate;

use std::time::Duration;

use ggez::{
    glam::Vec2,
    graphics::{Canvas, Color, DrawMode, DrawParam, Image, Rect},
    *,
};

const MARGIN: f32 = 5.0;
const BTN_SIZE: f32 = 60.0;
const BTN_SPACING: f32 = 70.0;

struct Button {
    rect: Rect,
    text: String,
    callback: fn() -> (),
}

impl Button {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if ctx
            .mouse
            .button_just_pressed(winit::event::MouseButton::Left)
        {
            let pos = ctx.mouse.position();

            if self.rect.contains(pos) {
                (self.callback)();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> Result<(), GameError> {
        let mesh =
            ggez::graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), self.rect, Color::GREEN)?;

        canvas.draw(&mesh, DrawParam::new());

        let text = ggez::graphics::Text::new(&self.text);

        canvas.draw(
            &text,
            Vec2::new(self.rect.x + 10.0, self.rect.center().y - 10.0),
        );

        Ok(())
    }
}

struct State {
    dt: Duration,
    buttons_left: Vec<Button>,
    buttons_right: Vec<Button>,
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let (w, _) = ctx.gfx.drawable_size();

        self.dt = ctx.time.delta();

        for btn in &mut self.buttons_left {
            btn.update(ctx)?;
        }
        for (i, btn) in self.buttons_right.iter_mut().enumerate() {
            btn.rect.x = w - (i + 1) as f32 * (btn.rect.w + 10.0);
            btn.update(ctx)?;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        let (w, _) = ctx.gfx.drawable_size();

        //let mouse_pos = ctx.mouse.position();

        let rect = ggez::graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0.0, 0.0, w, BTN_SIZE + 2.0 * MARGIN),
            Color::RED,
        )?;

        canvas.draw(&rect, DrawParam::new());

        let fps = 1.0 / self.dt.as_secs_f64();

        let text = ggez::graphics::Text::new(format!("{fps:.4}"));

        canvas.draw(&text, Vec2::new(100.0, 100.0));

        for btn in &mut self.buttons_left {
            btn.draw(ctx, &mut canvas)?;
        }
        for btn in &mut self.buttons_right {
            btn.draw(ctx, &mut canvas)?;
        }

        canvas.finish(ctx)?;

        Ok(())
    }
}

fn main() {
    let state = State {
        dt: std::time::Duration::new(0, 0),
        buttons_left: vec![
            Button {
                rect: Rect::new(5.0 + 0.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "+".to_owned(),
                callback: || println!("Botão + clicado"),
            },
            Button {
                rect: Rect::new(5.0 + 1.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "-".to_owned(),
                callback: || println!("Botão - clicado"),
            },
            Button {
                rect: Rect::new(5.0 + 2.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "Limpar".to_owned(),
                callback: || println!("Botão Limpar clicado"),
            },
        ],
        buttons_right: vec![
            Button {
                rect: Rect::new(5.0 + 0.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "dir1".to_owned(),
                callback: || println!("Botão dir1 clicado"),
            },
            Button {
                rect: Rect::new(5.0 + 1.0 * BTN_SPACING, 5.0, BTN_SIZE, BTN_SIZE),
                text: "dir2".to_owned(),
                callback: || println!("Botão dir2 clicado"),
            },
        ],
    };

    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("hello_ggez", "test")
        .default_conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state).unwrap()
}
