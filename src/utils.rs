use ggez::glam::Vec2;

pub fn remove_by_radius(vec: &mut Vec<[f32; 2]>, center: Vec2, radius: f32, changed: &mut bool) {
    let radius_sq = radius * radius;
    let mut i = 0;
    loop {
        if i >= vec.len() {
            break;
        }
        let pos = Vec2::from_array(vec[i]);
        if pos.distance_squared(center) <= radius_sq {
            vec.swap_remove(i);
            *changed = true;
        } else {
            i += 1;
        }
    }
}

pub fn paint_by_radius(
    movefrom: &mut Vec<[f32; 2]>,
    moveto: &mut Vec<[f32; 2]>,
    center: Vec2,
    radius: f32,
    changed: &mut bool,
) {
    let radius_sq = radius * radius;
    let mut i = 0;
    loop {
        if i >= movefrom.len() {
            break;
        }
        let pos = Vec2::from_array(movefrom[i]);
        if pos.distance_squared(center) <= radius_sq {
            moveto.push(pos.into());
            movefrom.swap_remove(i);
            *changed = true;
        } else {
            i += 1;
        }
    }
}
