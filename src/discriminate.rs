use ggez::{glam::Vec2, mint::Point2};
use good_lp::{
    Expression, ProblemVariables, Solution, SolverModel, clarabel, constraint, variable,
};
use nalgebra::{Const, DMatrix, DVector, Dyn, VecStorage};

pub type Point = [f32; 2];

pub struct LinearDiscrimination {
    pub vec_a: [f32; 2],
    pub scl_b: f32,
}

pub fn linear_discriminate(
    black_points: &[Point],
    white_points: &[Point],
) -> Result<LinearDiscrimination, String> {
    const DIMS: usize = 2;

    let n = black_points.len();
    let m = white_points.len();

    // u.len() == N
    // v.len() == M
    // a.len() == DIMS
    // b.len() == 1

    let mut variables = ProblemVariables::new();

    let u = variables.add_vector(variable().min(0), n);
    let v = variables.add_vector(variable().min(0), m);
    let a = variables.add_vector(variable(), DIMS);
    let b = variables.add(variable());

    let mut constraints = Vec::with_capacity(n + m);

    for i in 0..n {
        let ax: Expression = a.iter().zip(black_points[i]).map(|(a, b)| *a * b).sum();
        let c = constraint!(ax - b >= 1.0 - u[i]);
        constraints.push(c);
    }

    for j in 0..m {
        let ay: Expression = a.iter().zip(white_points[j]).map(|(a, b)| *a * b).sum();
        let c = constraint!(ay - b <= -(1.0 - v[j]));
        constraints.push(c);
    }

    let objective = u.iter().sum::<Expression>() + v.iter().sum::<Expression>();

    let solution = variables
        .minimise(objective)
        .using(clarabel)
        .with_all(constraints)
        .solve()
        .map_err(|e| format!("{e:?}"))?;

    let mut av = [0.0; DIMS];
    for i in 0..DIMS {
        av[i] = solution.value(a[i]) as f32;
    }
    let bv = solution.value(b) as f32;

    Ok(LinearDiscrimination {
        vec_a: av,
        scl_b: bv,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square() {
        let black_points = vec![[0.0, 0.0], [1.0, 0.0]];
        let white_points = vec![[0.0, 1.0], [1.0, 1.0]];

        let sol = linear_discriminate(&black_points, &white_points);

        match sol {
            Ok(_) => {}
            Err(err) => panic!("{err}"),
        }

        let Ok(sol) = sol else { unreachable!() };

        assert_eq!(sol.vec_a, [0.0, -2.0]);
        assert_eq!(sol.scl_b, -1.0);
    }

    #[test]
    fn test_diagonal() {
        let black_points = vec![[256.0, 289.0], [330.0, 216.0]];
        let white_points = vec![[323.0, 363.0], [410.0, 282.0], [488.0, 200.0]];

        let sol = linear_discriminate(&black_points, &white_points);

        match sol {
            Ok(_) => {}
            Err(err) => panic!("{err}"),
        }

        let Ok(sol) = sol else { unreachable!() };

        println!("a: {:?}\nb: {}", sol.vec_a, sol.scl_b);

        let exp_a = [-0.014104006229990487, -0.014277061521156017];
        let exp_b = -8.73816734446656;

        let eps = 1e-20;

        assert!((sol.vec_a[0] - exp_a[0]).abs() <= eps);
        assert!((sol.vec_a[1] - exp_a[1]).abs() <= eps);
        assert!((sol.scl_b - exp_b).abs() <= eps);
    }
}
