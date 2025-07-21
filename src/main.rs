use raylib::prelude::*;
use gjk_showcase::*;

const SIZE: i32 = 960;
const VERTEX_HANDLE_RADIUS: f32 = 20.0;
const SHAPE_HANDLE_RADIUS: f32 = 50.0;
const MAX_VERTICES: usize = 6;
const MIN_VERTICES: usize = 3;

const SHAPE1_COLOR: Color = Color::BLUE;
const SHAPE2_COLOR: Color = Color::GREEN;
const SIDE_LENGTH: f32 = 100.0;
const SHAPE1_POS: Vector2 = Vector2 {
    x: 100.0,
    y: 100.0,
};
const SHAPE2_POS: Vector2 = Vector2 {
    x: SHAPE1_POS.x + SIDE_LENGTH + SIDE_LENGTH,
    y: 100.0
};

const HELP_TEXT: &[&str] = &[
    "Drag Center: Move Shape",
    "Drag Corner: Move Vertex",
    "Up/Down: Add/Subtract vertex on blue shape",
    "Right/Left: Add/Subtract vertex on green shape",
];

//creates a square of points at the given position
fn make_square(vec: &mut Vec<Vector2>, pos: Vector2) {
    vec.clear();
    vec.push(pos);
    vec.push(pos + Vector2 { x: SIDE_LENGTH, y: 0.0});
    vec.push(pos + Vector2 { x: SIDE_LENGTH, y: SIDE_LENGTH });
    vec.push(pos + Vector2 { x: 0.0, y: SIDE_LENGTH });
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    Shape1,
    Shape2
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MoveTarget {
    None,
    Vertex{ idx: usize, shape: Shape },
    Shape(Shape),
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SIZE, SIZE)
        .title("GJK Algorithm Demo")
        .build();

    rl.set_exit_key(None);

    let mut shape1 = vec![];
    make_square(&mut shape1, SHAPE1_POS);

    let mut shape2 = vec![];
    make_square(&mut shape2, SHAPE2_POS);

    let mut moving = MoveTarget::None;

    while !rl.window_should_close() {
        let mouse_pos = rl.get_mouse_position();
        let mouse_down = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);

        let mut shape1_avg = Vector2::zero();
        let mut shape2_avg = Vector2::zero();
        let mut shape_handle_pos = Vector2::zero();

        let target = {
            let mut vertex_idx = 0;
            let mut nearest_dist = (shape1[0] - mouse_pos).length();
            shape1_avg += shape1[0];

            let mut vertex_shape = Shape::Shape1;
            for i in 1..shape1.len() {
                let dist = (shape1[i] - mouse_pos).length();
                if dist < nearest_dist {
                    vertex_idx = i;
                    nearest_dist = dist;
                }
                shape1_avg += shape1[i];
            }
            shape1_avg *= 1.0 / shape1.len() as f32;

            for i in 0..shape2.len() {
                let dist = (shape2[i] - mouse_pos).length();
                if dist < nearest_dist {
                    vertex_idx = i;
                    nearest_dist = dist;
                    vertex_shape = Shape::Shape2;
                }
                shape2_avg += shape2[i];
            }
            shape2_avg *= 1.0 / shape2.len() as f32;

            if moving == MoveTarget::None {
                let shape1_handle_dist = (shape1_avg - mouse_pos).length();
                let shape2_handle_dist = (shape2_avg - mouse_pos).length();
                let (near_shape, near_shape_dist) = if shape1_handle_dist < shape2_handle_dist {
                    shape_handle_pos = shape1_avg;
                    (Shape::Shape1, shape1_handle_dist)
                } else {
                    shape_handle_pos = shape2_avg;
                    (Shape::Shape2, shape2_handle_dist)
                };

                if nearest_dist <= VERTEX_HANDLE_RADIUS {
                    MoveTarget::Vertex { idx: vertex_idx, shape: vertex_shape }
                } else if near_shape_dist <= SHAPE_HANDLE_RADIUS {
                    MoveTarget::Shape(near_shape)
                } else {
                    MoveTarget::None
                }
            } else {
                MoveTarget::None
            }
        };

        let draw_handle = if mouse_down {
            if moving == MoveTarget::None {
                moving = target;
            }
            None
        } else {
            moving = MoveTarget::None;
            match target {
                MoveTarget::None => None,
                MoveTarget::Vertex { idx, shape } => {
                    if shape == Shape::Shape2 {
                        Some( (shape2[idx], VERTEX_HANDLE_RADIUS, SHAPE2_COLOR) )
                    } else {
                        Some( (shape1[idx], VERTEX_HANDLE_RADIUS, SHAPE1_COLOR) )
                    }
                },
                MoveTarget::Shape(shape) => {
                    if shape == Shape::Shape1 {
                        Some( (shape_handle_pos, SHAPE_HANDLE_RADIUS, SHAPE1_COLOR) )
                    } else {
                        Some( (shape_handle_pos, SHAPE_HANDLE_RADIUS, SHAPE2_COLOR) )
                    }
                }
            }
        };

        if let MoveTarget::Vertex{shape, idx} = moving {
            if shape == Shape::Shape1 {
                shape1[idx] = mouse_pos;
            } else {
                shape2[idx] = mouse_pos;
            }
        } else if let MoveTarget::Shape(shape) = moving {
            if shape == Shape::Shape1 {
                let mut offsets = vec![];
                for point in shape1.iter() {
                    offsets.push(*point - shape1_avg);
                }
                for point in shape1.iter_mut().zip(offsets.iter()) {
                    *point.0 = *point.1 + mouse_pos;
                }
            } else {
                let avg_offset = mouse_pos - shape2_avg;
                for point in shape2.iter_mut() {
                    *point += avg_offset;
                }
            }
        } else {
            if rl.is_key_pressed(KeyboardKey::KEY_R) {
                make_square(&mut shape1, SHAPE1_POS);
                make_square(&mut shape2, SHAPE2_POS);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_UP) && shape1.len() < MAX_VERTICES {
                shape1.push(shape1_avg);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_DOWN) && shape1.len() > MIN_VERTICES {
                shape1.pop();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) && shape2.len() < MAX_VERTICES {
                shape2.push(shape2_avg);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT) && shape2.len() > MIN_VERTICES {
                shape2.pop();
            }
        }

        //perform rendering
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            let mut y = 12;
            for text in HELP_TEXT {
                d.draw_text(text, 12, y, 20, Color::DARKGRAY);
                y += 20;
            }

            d.draw_circle(SIZE / 2, SIZE / 2, 3.0, Color::WHITE);

            for i in 1..shape1.len() {
                d.draw_line_v(shape1[i - 1], shape1[i], SHAPE1_COLOR);
            }
            d.draw_line_v(shape1[0], shape1.last().unwrap(), SHAPE1_COLOR);

            for i in 1..shape2.len() {
                d.draw_line_v(shape2[i - 1], shape2[i], SHAPE2_COLOR);
            }
            d.draw_line_v(shape2[0], shape2.last().unwrap(), SHAPE2_COLOR);

            if let Some(handle) = draw_handle {
                d.draw_circle_lines_v(handle.0, handle.1 as f32, handle.2);
            }

            if intersects_2d(&mut d, &shape1, &shape2) {
                d.draw_rectangle_lines_ex(
                    Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: SIZE as f32,
                        height: SIZE as f32
                    },
                    4.0,
                    Color::RED
                );
            }
        }
    }
}
