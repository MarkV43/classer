use std::time::Duration;

use ggez::{
    glam::Vec2,
    graphics::{Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, Rect},
    *,
};

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
    btn: Button,
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.dt = ctx.time.delta();

        self.btn.update(ctx)?;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        let (w, _) = ctx.gfx.drawable_size();

        let mouse_pos = ctx.mouse.position();

        let rect = ggez::graphics::Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0.0, 0.0, w, mouse_pos.y),
            Color::RED,
        )?;

        canvas.draw(&rect, DrawParam::new());

        let fps = 1.0 / self.dt.as_secs_f64();

        let text = ggez::graphics::Text::new(format!("{fps:.4}"));

        canvas.draw(&text, Vec2::new(100.0, 100.0));

        self.btn.draw(ctx, &mut canvas)?;

        canvas.finish(ctx)?;

        Ok(())
    }
}

fn main() {
    let state = State {
        dt: std::time::Duration::new(0, 0),
        btn: Button {
            rect: Rect::new(5.0, 5.0, 40.0, 40.0),
            text: "+".to_owned(),
            callback: || println!("Botão clicado"),
        },
    };

    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("hello_ggez", "test")
        .default_conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state).unwrap()
}
