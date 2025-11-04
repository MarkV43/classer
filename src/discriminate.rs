use convex_opt::linear::{LinearProblem, simplex::simplex};
use nalgebra::{Const, DMatrix, DVector, Dyn, VecStorage};

pub type Point = [f64; 2];

pub struct LinearDiscrimination {
    vec_a: [f64; 2],
    scl_b: f64,
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

    let vars = n + m + DIMS + 1;

    let mut objective = vec![1.0; vars];
    objective[n + m..].fill(0.0);

    let mut mat_a = DMatrix::zeros(2 * (n + m), vars);
    let mut vec_b = DVector::zeros(2 * (n + m));
    let vec_c = DVector::from_data(VecStorage::new(Dyn(objective.len()), Const, objective));

    // a^T x_i - b >= 1 - u_i, i = 1,...,N
    // a^T x_i - b + u_i >= 1, i = 1,...,N
    for (i, pnt) in black_points.iter().enumerate() {
        let mut coefs = mat_a.row_mut(i);

        coefs[i] = 1.0;
        coefs[vars - 1] = -1.0;
        coefs
            .columns_range_mut(n + m..n + m + DIMS)
            .copy_from_slice(pnt);

        vec_b[i] = 1.0;
    }

    // a^T y_i - b <= -(1 - v_i), i = 1,...,M
    // a^T y_i - b - v_i <= -1, i = 1,...,M
    // -a^T y_i + b + v_i >= 1, i = 1,...,M
    for (i, pnt) in white_points.iter().enumerate() {
        let mut coefs = mat_a.row_mut(n + i);
        coefs[n + i] = 1.0;
        coefs[vars - 1] = 1.0;
        coefs
            .columns_range_mut(n + m..n + m + DIMS)
            .iter_mut()
            .zip(pnt.iter())
            .for_each(|(a, b)| *a = -*b);

        vec_b[n + i] = 1.0;
    }

    // u >= 0, v >= 0
    for i in 0..n + m {
        let mut coefs = mat_a.row_mut(n + m + i);
        coefs[i] = 1.0;

        vec_b[n + m + i] = 0.0;
    }

    let lp = LinearProblem::new_gt_unb(
        convex_opt::linear::OptimizationKind::Min,
        mat_a,
        vec_b,
        vec_c,
    );

    println!("{lp}");

    let (obj, all_x) = simplex(&lp, 1e-12);

    let xp = all_x.view((0, 0), (vars, 1));
    let xn = all_x.view((vars, 0), (vars, 1));

    let x = xp - xn;
    let xs = x.as_slice();

    let a = xs[n + m..n + m + DIMS].try_into().unwrap();
    let b = xs[n + m + DIMS];

    Ok(LinearDiscrimination { vec_a: a, scl_b: b })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_discrimination() {
        let black_points = vec![[0.0, 0.0], [1.0, 0.0]];
        let white_points = vec![[0.0, 1.0], [1.0, 1.0]];

        let sol = linear_discriminate(&black_points, &white_points);

        match sol {
            Ok(_) => {}
            Err(err) => panic!("{err}"),
        }

        let Ok(sol) = sol else { unreachable!() };

        println!("vec_a: {:?}\nscl_b: {}", sol.vec_a, sol.scl_b);

        todo!()
    }
}
