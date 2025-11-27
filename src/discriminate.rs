use classer_opt::{BarrierParams, LineSearchParams, NewtonParams, linear::LinearDP};
use crevice::std140::AsStd140;
use ggez::glam::Vec2;
use nalgebra::Vector2;

pub type Point = [f32; 2];

#[derive(AsStd140, Clone, Debug)]
pub struct LinearDiscrimination {
    pub vec_a: Vec2,
    pub scl_b: f32,
}

impl LinearDiscrimination {
    pub fn none() -> Self {
        Self {
            vec_a: Vec2::new(0.0, 0.0),
            scl_b: 0.0,
        }
    }
}

pub fn linear_discriminate(
    black_points: &[[f32; 2]],
    white_points: &[[f32; 2]],
) -> Result<LinearDiscrimination, String> {
    let mut problem = LinearDP {
        xs: black_points
            .iter()
            .map(|x| Vector2::new(x[0] as f64, x[1] as f64))
            .collect(),
        ys: white_points
            .iter()
            .map(|y| Vector2::new(y[0] as f64, y[1] as f64))
            .collect(),
        gamma: 1.0,
    };

    let lsp = LineSearchParams::new(0.3, 0.7);

    let center = NewtonParams::new(1e-8, lsp.clone(), 128, 1024);
    let params = BarrierParams::new(1e-3, 10.0, 1e-8, center);

    let aux_center = NewtonParams::new(1e-5, lsp, 16, 32);
    let aux_params = BarrierParams::new(1e-3, 10.0, 1e-1, aux_center);

    let (a, b) = problem.solve(&params, &aux_params)?;

    Ok(LinearDiscrimination {
        vec_a: Vec2::new(a[0] as f32, a[1] as f32),
        scl_b: b as f32,
    })
}
