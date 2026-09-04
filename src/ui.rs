use raylib::prelude::*;
pub fn draw_health_bar(
    d: &mut RaylibDrawHandle,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hp_points: i32,
    max_hp_points: i32,
) {
    let ratio = if max_hp_points > 0 {
        hp_points as f32 / max_hp_points as f32
    } else {
        0.0
    };
    let ratio = ratio.clamp(0.0, 1.0);

    // fond (barre vide)
    d.draw_rectangle(
        x as i32,
        y as i32,
        width as i32,
        height as i32,
        Color::new(60, 60, 60, 255),
    );

    // remplissage (HP actuels)
    let fill_width = (width * ratio) as i32;
    d.draw_rectangle(
        x as i32,
        y as i32,
        fill_width,
        height as i32,
        Color::new(200, 30, 30, 255),
    );

    // contour
    d.draw_rectangle_lines(
        x as i32,
        y as i32,
        width as i32,
        height as i32,
        Color::BLACK,
    );
}
