use classer_opt::linear::LinearDP;
use crevice::std140::AsStd140;
use ggez::glam::Vec2;
use ipm::alg::{barrier::BarrierParams, line_search::LineSearchParams, newton::NewtonParams};
use nalgebra::Vector2;

pub type Point = [f32; 2];

#[derive(AsStd140, Clone)]
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

pub fn linear_discriminate(black_points: &[Point], white_points: &[Point]) -> LinearDiscrimination {
    let mut problem = LinearDP {
        xs: black_points
            .iter()
            .map(|x| Vector2::new(x[0], x[1]))
            .collect(),
        ys: white_points
            .iter()
            .map(|y| Vector2::new(y[0], y[1]))
            .collect(),
        gamma: 1.0,
    };

    let lsp = LineSearchParams::new(0.3, 0.6);
    let center = NewtonParams::new(1e-1, lsp, 8, 20);
    let params = BarrierParams::new(10.0, 30.0, 1e-3, center);

    let (a, b) = problem.solve(&params);

    LinearDiscrimination {
        vec_a: Vec2::from_array(a),
        scl_b: b,
    }
}
