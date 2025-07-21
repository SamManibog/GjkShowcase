use raylib::prelude::*;
pub const GJK_MAX_ITERATIONS: u32 = 100;

///a shape that the gjk algorithm can be ran on
///implementer must ensure that all instances are convex shapes
pub trait ConvexShape2D {
    ///Must return the furthest point on the edge of the shape
    ///This will be at the point where the dot product between the shape and the direction are at
    ///their greatest.
    fn furthest_point(&self, direction: Vector2) -> Vector2;
}

impl ConvexShape2D for Vec<Vector2> {
    fn furthest_point(&self, direction: Vector2) -> Vector2 {
        let mut max = self[0];
        let mut max_dot = max.dot(direction);

        for i in 1..self.len() {
            let dot = self[i].dot(direction);
            if dot > max_dot {
                max = self[i];
                max_dot = dot;
            }
        }

        max
    }
}

pub fn intersects_2d<S, T>(d: &mut RaylibDrawHandle,shape1: &S, shape2: &T) -> bool
where S: ConvexShape2D, T: ConvexShape2D {

    //let mut direction = shape1.center() - shape2.center();
    let mut direction = Vector2::one().normalized();
    let mut simplex = vec![minowski_support_2d(shape1, shape2, direction)];

    let modvec = vec![
        Vector2{x:5.0,y:-10.0},
        Vector2{x:5.0,y:5.0},
        Vector2{x:-10.0,y:5.0},
        Vector2{x:-10.0,y:-10.0}
    ];
    {
        let o = Vector2 {
            x: 480.0,
            y: 480.0
        };

        let line = Vector2 {
            x: o.x + direction.x * 100.0,
            y: o.y + direction.y * 100.0
        };

        let pixel = o + simplex[0];
        let pixel_label = pixel + modvec[0].scale_by(2.0);

        d.draw_line_v(o, line, Color::WHITE);
        let text = "0";
        d.draw_text(&text, line.x as i32 + 10, line.y as i32 + 10, 16, Color::WHITE);

        d.draw_circle_v(pixel, 2.0,  Color::CYAN);
        d.draw_text(&text, pixel_label.x as i32, pixel_label.y as i32, 16, Color::CYAN);
    }

    //support point for vector in direction of origin
    direction = -simplex[0].normalized();

    for i in 1..=GJK_MAX_ITERATIONS {
        let point = minowski_support_2d(shape1, shape2, direction);
        {
            let o = Vector2 {
                x: 480.0,
                y: 480.0
            };


            let pixel = o + point;
            let line = o + direction.scale_by(100.0);
            let pixel_label = pixel + modvec[(i%4) as usize].scale_by(2.0);

            d.draw_line_v(o, line, Color::WHITE);
            let text = format!("{}", i);
            d.draw_text(&text, line.x as i32 + 10, line.y as i32 + 10, 16, Color::WHITE);

            d.draw_circle_v(pixel, 2.0,  Color::CYAN);
            d.draw_text(&text, pixel_label.x as i32, pixel_label.y as i32, 16, Color::CYAN);
        }
        if point.dot(direction) < 0.0 {
            return false;
        };
        simplex.push(point);

        if match simplex.len() {
            2 => handle_simplex_1d(&mut simplex, &mut direction),
            3 => handle_simplex_2d(d, &mut simplex, &mut direction),
            _ => unreachable!()
        } { return true };
    }
    return false;
}

//takes in a simplex and direction, then returns false
//the provided simplex must have at least two points
fn handle_simplex_1d(simplex: &mut Vec<Vector2>, direction: &mut Vector2) -> bool {
    let a = simplex[1];
    let b = simplex[0];
    *direction = origin_normal_2d(a, b).normalized();
    false
}

//takes in a simplex and direction, then returns true if the point is contained in the simplex
//the provided simplex must have at least three points
fn handle_simplex_2d(d: &mut RaylibDrawHandle, simplex: &mut Vec<Vector2>, direction: &mut Vector2) -> bool {
    let a = simplex[2];
    let b = simplex[1];
    let c = simplex[0];

    let norm_ab = normal_2d(a, b);
    let norm_ac = normal_2d(a, c);
    let dir_ab = norm_ab.scale_by(norm_ab.dot(a - c).signum()).normalized();
    let dir_ac = norm_ac.scale_by(norm_ac.dot(a - b).signum()).normalized();

    {
        let o = Vector2 {
            x: 480.0,
            y: 480.0
        };

        d.draw_line_v(a + o, c + o, Color::VIOLET);
        d.draw_line_v(b + o, c + o, Color::VIOLET);
        d.draw_line_v(a + o, b + o, Color::VIOLET);
    }

    if dir_ab.dot(a) < 0.0 {
        simplex.remove(0);
        *direction = dir_ab;
        false
    } else if dir_ac.dot(a) < 0.0 {
        simplex.remove(1);
        *direction = dir_ac;
        false
    } else {
        true
    }
}

//returns the normal of two vectors
fn normal_2d(v1: Vector2, v2: Vector2) -> Vector2 {
    Vector2 {
        x: v1.y - v2.y,
        y: v2.x - v1.x
    }
}

//returns a normal of the two vectors that faces the origin
fn origin_normal_2d(v1: Vector2, v2: Vector2) -> Vector2 {
    let normal = normal_2d(v1, v2);
    normal.scale_by(-normal.dot(v1).signum())
}

//function finding the support point of the minowski difference in a direction
fn minowski_support_2d<S: ConvexShape2D, T: ConvexShape2D>(s1: &S, s2: &T, direction: Vector2) -> Vector2 {
    s1.furthest_point(direction) - s2.furthest_point(-direction)
}
