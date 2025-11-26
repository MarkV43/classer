use ggez::{
    Context, GameError,
    glam::Vec2,
    graphics::{self, Canvas, Color, DrawMode, DrawParam, Image, Rect},
};

use crate::constants::{DARK_GRAY, LIGHT_GRAY};

pub struct Button {
    pub rect: Rect,
    pub text: String,
    pub image: Image,
}

impl Button {
    pub fn update(
        &mut self,
        ctx: &mut Context,
        callback: impl FnOnce() -> (),
    ) -> Result<(), GameError> {
        if ctx
            .mouse
            .button_just_pressed(ggez::winit::event::MouseButton::Left)
        {
            let pos = ctx.mouse.position();

            if self.rect.contains(pos) {
                (callback)();
            }
        }

        Ok(())
    }

    pub fn draw(
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

        let scale = Vec2::new(0.1, 0.1);
        let dst = center - 0.5 * scale * img_size;

        canvas.draw(
            &self.image,
            graphics::DrawParam::new().dest(dst).scale(scale),
        );

        Ok(())
    }
}
