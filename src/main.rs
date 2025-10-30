use ggez::{
    glam::Vec2,
    graphics::{Canvas, Color, Drawable},
    *,
};

struct State {
    dt: std::time::Duration,
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.dt = ctx.time.delta();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        let text = ggez::graphics::Text::new(format!("{:?}", self.dt));

        canvas.draw(&text, Vec2::new(100.0, 100.0));

        canvas.finish(ctx)?;

        Ok(())
    }
}

fn main() {
    let state = State {
        dt: std::time::Duration::new(0, 0),
    };

    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("hello_ggez", "test")
        .default_conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state).unwrap()
}
