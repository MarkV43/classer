use classer_opt::linear::LinearDP;
use crevice::std140::AsStd140;
use ggez::glam::Vec2;
use ipm::alg::{
    descent::newton::NewtonsMethod,
    ipm::{barrier::BarrierMethod, infeasible::InfeasibleIpm},
    line_search::guarded::GuardedLineSearch,
};
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

    let lsp = GuardedLineSearch::new(0.3, 0.7).unwrap();

    let center = NewtonsMethod::new(1e-8, lsp.clone(), 128, 1024).unwrap();
    let params = BarrierMethod::new(1e-1, 10.0, 1e-8, center).unwrap();

    let aux_center = NewtonsMethod::new(1e-5, lsp, 16, 32).unwrap();
    let aux_params = BarrierMethod::new(1e-1, 1.5, 1e-3, aux_center).unwrap();

    let inf_ipm = InfeasibleIpm::new(aux_params, params);

    let (a, b) = problem.solve(&inf_ipm)?;

    Ok(LinearDiscrimination {
        vec_a: Vec2::new(a[0] as f32, a[1] as f32),
        scl_b: b as f32,
    })
}
