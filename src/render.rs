use crate::animation::Animation;
use crate::assets::Assets;
use crate::cursor::CursorType;
use crate::duel::{DuelConditions, DuelSide};
use crate::game_mode::{GameMode, TurnPhase};
use crate::grid::DuelPreview;
use crate::map::TileMap;
use crate::minigame::{MiniGame, TimingResult};
use crate::screens::{guide, winfo};
use crate::ui::{self, HudRects};
use crate::unit::{Unit, UnitState};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE};
use raylib::prelude::*;

fn unit_source_rec(animation: &Animation, unit: &Unit) -> Rectangle {
    let mut source_rec = animation.animation_frame();
    if unit.facing_left {
        source_rec.width = -source_rec.width;
    }
    source_rec
}

fn draw_texture_at(d: &mut RaylibDrawHandle, texture: &Texture2D, rect: Rectangle) {
    d.draw_texture_ex(
        texture,
        Vector2::new(rect.x, rect.y),
        0.0,
        1.0,
        Color::WHITE,
    );
}

/// avance l'animation puis la dessine
fn draw_unit_sprite(
    d: &mut RaylibDrawHandle,
    animation: &mut Animation,
    unit: &Unit,
    x: f32,
    y: f32,
    size: f32,
    delta_time: f32,
) {
    animation.animation_update(delta_time);
    d.draw_texture_pro(
        &animation.texture,
        unit_source_rec(animation, unit),
        Rectangle {
            x,
            y,
            width: size,
            height: size,
        },
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE,
    );
}

// Title screen ------------------------------------------------------------------
pub fn draw_title_screen(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    hud: &HudRects,
    mouse_position: Vector2,
) {
    d.clear_background(Color::BLACK);

    draw_fullscreen_texture(d, &assets.title_background_texture);

    draw_texture_at(d, &assets.btn_play_title_screen, hud.btn_play_title_screen);
    draw_texture_at(
        d,
        &assets.btn_multi_title_screen,
        hud.btn_multi_title_screen,
    );
    draw_texture_at(
        d,
        &assets.btn_settings_title_screen,
        hud.btn_settings_title_screen,
    );
    draw_texture_at(d, &assets.btn_exit_title_screen, hud.btn_exit_title_screen);
    draw_texture_at(
        d,
        &assets.btn_guide_title_screen,
        hud.btn_learn_title_screen,
    );

    d.draw_texture_ex(
        assets.cursor_texture(CursorType::Normal),
        Vector2::new(mouse_position.x, mouse_position.y),
        0.0,
        0.7,
        Color::WHITE,
    );
}

// Pause screen --------------------------------------------------------------------
pub fn draw_pause_screen(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    hud: &HudRects,
    mouse_position: Vector2,
) {
    draw_fullscreen_texture(d, &assets.menu_background_texture);

    let title = "Pause";
    let title_size = 48.0;
    let title_text_size = assets.hud_font.measure_text(title, title_size, 1.0);
    let title_pos = Vector2::new(
        SCREEN_WIDTH as f32 / 2.0 - title_text_size.x / 2.0,
        hud.btn_pause_play.y - title_size - 30.0,
    );
    // petite ombre pour rester lisible sur un ciel clair
    d.draw_text_ex(
        &assets.hud_font,
        title,
        title_pos + Vector2::new(2.0, 2.0),
        title_size,
        1.0,
        Color::new(0, 0, 0, 160),
    );
    d.draw_text_ex(
        &assets.hud_font,
        title,
        title_pos,
        title_size,
        1.0,
        Color::WHITE,
    );

    for (texture, rect) in [
        (&assets.btn_play, hud.btn_pause_play),
        (&assets.btn_guide, hud.btn_pause_guide),
        (&assets.btn_settings, hud.btn_pause_settings),
        (&assets.btn_back, hud.btn_pause_back),
        (&assets.btn_exit, hud.btn_pause_exit),
    ] {
        draw_texture_at(d, texture, rect);
    }

    d.draw_texture_ex(
        assets.cursor_texture(CursorType::Normal),
        Vector2::new(mouse_position.x, mouse_position.y),
        0.0,
        0.7,
        Color::WHITE,
    );
}

// Guide screen --------------------------------------------------------------------
pub fn draw_guide_screen(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    delta_time: f32,
    hud: &HudRects,
    mouse_position: Vector2,
    scroll: f32,
) {
    draw_fullscreen_texture(d, &assets.menu_background_texture);

    let title = "How to play";
    let title_size = 48.0;
    let title_text_size = assets.hud_font.measure_text(title, title_size, 1.0);
    let title_pos = Vector2::new(
        SCREEN_WIDTH as f32 / 2.0 - title_text_size.x / 2.0,
        guide::HEADER_HEIGHT / 2.0 - title_text_size.y / 2.0,
    );
    d.draw_text_ex(
        &assets.hud_font,
        title,
        title_pos + Vector2::new(2.0, 2.0),
        title_size,
        1.0,
        Color::new(0, 0, 0, 160),
    );
    d.draw_text_ex(
        &assets.hud_font,
        title,
        title_pos,
        title_size,
        1.0,
        Color::WHITE,
    );

    let sprite_size = 190.0;
    let sprite_top = guide::HEADER_HEIGHT / 2.0 - sprite_size / 2.0 + 10.0;
    let sprite_offset = title_text_size.x / 2.0 + 90.0;
    draw_idle_card(
        d,
        &mut assets.cavalry.idle,
        SCREEN_WIDTH as f32 / 2.0 - sprite_offset,
        sprite_top,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.necromancer.idle,
        SCREEN_WIDTH as f32 / 2.0 + sprite_offset,
        sprite_top,
        sprite_size,
        delta_time,
    );

    let panel = guide::panel_rect(hud);
    d.draw_rectangle_rec(panel, Color::new(20, 20, 20, 210));
    d.draw_rectangle_lines_ex(panel, 2.0, Color::new(255, 255, 255, 90));

    // contenu qui scrolle, coupé aux bords du panneau
    {
        let mut clip = d.begin_scissor_mode(
            panel.x as i32,
            panel.y as i32,
            panel.width as i32,
            panel.height as i32,
        );
        let x = panel.x + guide::PANEL_PAD;
        let width = panel.width - guide::PANEL_PAD * 2.0 - guide::SCROLLBAR_WIDTH;
        let mut y = panel.y + guide::PANEL_PAD - scroll;
        for block in guide::content() {
            let height = block.height();
            // on ne dessine que ce qui est visible
            if y + block.draw_extent() >= panel.y && y <= panel.y + panel.height {
                draw_guide_block(&mut clip, assets, &block, x, y, width);
            }
            y += height;
        }
    }

    // scrollbar (seulement si tout ne tient pas dans le panneau)
    if guide::max_scroll(hud) > 0.0 {
        d.draw_rectangle_rec(guide::scrollbar_track(hud), Color::new(255, 255, 255, 40));
        d.draw_rectangle_rec(
            guide::scrollbar_thumb(hud, scroll),
            Color::new(255, 255, 255, 170),
        );
    }

    ui::draw_text_button(d, &assets.hud_font, hud.btn_guide_back, "Back");

    d.draw_texture_ex(
        assets.cursor_texture(CursorType::Normal),
        Vector2::new(mouse_position.x, mouse_position.y),
        0.0,
        0.7,
        Color::WHITE,
    );
}

fn draw_guide_block(
    d: &mut impl RaylibDraw,
    assets: &Assets,
    block: &guide::Block,
    x: f32,
    y: f32,
    width: f32,
) {
    match block {
        guide::Block::Heading(title) => {
            d.draw_text_ex(
                &assets.hud_font,
                title,
                Vector2::new(x, y + 8.0),
                guide::HEADING_SIZE,
                1.0,
                Color::GOLD,
            );
            d.draw_rectangle(
                x as i32,
                (y + 8.0 + guide::HEADING_SIZE + 2.0) as i32,
                width as i32,
                2,
                Color::new(255, 203, 0, 120),
            );
        }
        guide::Block::Text(line) => {
            d.draw_text_ex(
                &assets.info_font,
                line,
                Vector2::new(x, y),
                guide::TEXT_SIZE,
                1.0,
                Color::WHITE,
            );
        }
        guide::Block::Gap => {}
        guide::Block::Diagram { rows, caption } => {
            draw_guide_diagram(d, assets, rows, x, y);
            let columns = rows
                .iter()
                .map(|row| row.chars().filter(|c| !c.is_whitespace()).count())
                .max()
                .unwrap_or(0);
            // légendes alignées sur une même colonne (min 3 cases de large)
            let caption_x = x + columns.max(3) as f32 * guide::DIAGRAM_CELL + 24.0;
            for (i, line) in caption.iter().enumerate() {
                d.draw_text_ex(
                    &assets.info_font,
                    line,
                    Vector2::new(caption_x, y + 4.0 + i as f32 * 24.0),
                    guide::CAPTION_SIZE,
                    1.0,
                    if i == 0 { Color::WHITE } else { guide::MUTED },
                );
            }
        }
        guide::Block::Bar {
            window,
            precision,
            cursor,
            lives,
            caption,
        } => {
            let lives_space = 50.0;
            let bar_x = x + lives_space;
            let bar_y = y + 12.0;
            let bar_width = 300.0;
            let bar_height = 22.0;
            d.draw_rectangle(
                bar_x as i32,
                bar_y as i32,
                bar_width as i32,
                bar_height as i32,
                Color::DARKGRAY,
            );
            // zones centrées dans la barre d'exemple
            let zone_x = bar_x + (0.5 - window / 2.0) * bar_width;
            d.draw_rectangle(
                zone_x as i32,
                bar_y as i32,
                (window * bar_width) as i32,
                bar_height as i32,
                Color::YELLOW,
            );
            let perfect_x = bar_x + (0.5 - precision / 2.0) * bar_width;
            d.draw_rectangle(
                perfect_x as i32,
                bar_y as i32,
                (precision * bar_width).ceil() as i32,
                bar_height as i32,
                Color::GREEN,
            );
            d.draw_rectangle(
                (bar_x + cursor * bar_width) as i32 - 2,
                bar_y as i32 - 4,
                4,
                bar_height as i32 + 8,
                Color::WHITE,
            );
            for i in 0..*lives {
                d.draw_rectangle(
                    (bar_x - 20.0 - i as f32 * 16.0) as i32,
                    (bar_y + 5.0) as i32,
                    12,
                    12,
                    Color::RED,
                );
            }
            d.draw_text_ex(
                &assets.info_font,
                caption,
                Vector2::new(bar_x + bar_width + 20.0, bar_y + 1.0),
                guide::CAPTION_SIZE,
                1.0,
                guide::MUTED,
            );
        }
        guide::Block::Panel(lines) => {
            let rect = Rectangle::new(
                x,
                y + 6.0,
                width.min(700.0),
                lines.len() as f32 * 22.0 + 16.0,
            );
            d.draw_rectangle_rec(rect, Color::new(10, 10, 10, 235));
            d.draw_rectangle_lines_ex(rect, 2.0, guide::GOOD);
            for (i, (line, color)) in lines.iter().enumerate() {
                d.draw_text_ex(
                    &assets.info_font,
                    line,
                    Vector2::new(rect.x + 10.0, rect.y + 8.0 + i as f32 * 22.0),
                    17.0,
                    1.0,
                    *color,
                );
            }
        }
        guide::Block::Illustration(kind) => {
            let rect = Rectangle::new(
                x + width - guide::ILLUSTRATION_WIDTH,
                y,
                guide::ILLUSTRATION_WIDTH,
                guide::ILLUSTRATION_HEIGHT,
            );
            draw_guide_illustration(d, kind, rect);
        }
    }
}

/// mini grille du guide : jetons bleus pour ton camp, rouges pour l'ennemi,
/// cases jaunes avec badge pour les choix de case
fn draw_guide_diagram(d: &mut impl RaylibDraw, assets: &Assets, rows: &[&str], x: f32, y: f32) {
    let cell = guide::DIAGRAM_CELL;
    for (row_i, row) in rows.iter().enumerate() {
        for (col_i, c) in row.chars().filter(|c| !c.is_whitespace()).enumerate() {
            let rect = Rectangle::new(x + col_i as f32 * cell, y + row_i as f32 * cell, cell, cell);
            d.draw_rectangle_rec(rect, Color::new(70, 95, 60, 255));
            d.draw_rectangle_lines_ex(rect, 1.0, Color::new(0, 0, 0, 90));

            let center = Vector2::new(rect.x + cell / 2.0, rect.y + cell / 2.0);
            let token = match c {
                'A' => Some((Color::new(70, 130, 255, 255), true)),
                'a' => Some((Color::new(70, 130, 255, 255), false)),
                'E' => Some((Color::new(220, 60, 60, 255), true)),
                'e' => Some((Color::new(220, 60, 60, 255), false)),
                _ => None,
            };
            if let Some((color, main)) = token {
                d.draw_circle_v(center, cell * 0.38, color);
                if main {
                    d.draw_ring(
                        center,
                        cell * 0.38,
                        cell * 0.38 + 3.0,
                        0.0,
                        360.0,
                        24,
                        Color::WHITE,
                    );
                }
                let label = c.to_string();
                let size = assets.info_font.measure_text(&label, 18.0, 1.0);
                d.draw_text_ex(
                    &assets.info_font,
                    &label,
                    Vector2::new(center.x - size.x / 2.0, center.y - size.y / 2.0),
                    18.0,
                    1.0,
                    Color::WHITE,
                );
            }

            let badge = match c {
                '2' => Some(("+25", guide::GOOD)),
                '1' => Some(("+12", guide::GOOD)),
                '0' => Some(("0", Color::WHITE)),
                'n' => Some(("-4", guide::BAD)),
                _ => None,
            };
            if let Some((label, color)) = badge {
                d.draw_rectangle_rec(rect, Color::new(255, 255, 0, 170));
                let size = assets.info_font.measure_text(label, 13.0, 1.0);
                d.draw_rectangle(
                    rect.x as i32 + 1,
                    rect.y as i32 + 2,
                    size.x as i32 + 4,
                    16,
                    Color::new(20, 20, 20, 200),
                );
                d.draw_text_ex(
                    &assets.info_font,
                    label,
                    Vector2::new(rect.x + 3.0, rect.y + 3.0),
                    13.0,
                    1.0,
                    color,
                );
            }
        }
    }
}

fn draw_fullscreen_texture(d: &mut RaylibDrawHandle, texture: &Texture2D) {
    d.draw_texture_pro(
        texture,
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: texture.width as f32,
            height: texture.height as f32,
        },
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: SCREEN_WIDTH as f32,
            height: SCREEN_HEIGHT as f32,
        },
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE,
    );
}

// Faction selection screen ------------------------------------------------------
fn draw_idle_card(
    d: &mut RaylibDrawHandle,
    animation: &mut Animation,
    center_x: f32,
    top_y: f32,
    sprite_size: f32,
    delta_time: f32,
) {
    animation.animation_update(delta_time);
    let frame = animation.animation_frame();
    // garde le ratio d'origine de la frame pour ne pas déformer le sprite
    let scale = (sprite_size / frame.width).min(sprite_size / frame.height);
    let dest_width = frame.width * scale;
    let dest_height = frame.height * scale;
    d.draw_texture_pro(
        &animation.texture,
        frame,
        Rectangle {
            x: center_x - dest_width / 2.0,
            y: top_y + (sprite_size - dest_height) / 2.0,
            width: dest_width,
            height: dest_height,
        },
        Vector2::new(0.0, 0.0),
        0.0,
        Color::WHITE,
    );
}

/// centres des 3 colonnes de sprites pour une faction, centrées sur `center_x`.
/// `col_pitch` est l'espacement entre centres
fn faction_column_centers(center_x: f32, col_pitch: f32) -> [f32; 3] {
    [center_x - col_pitch, center_x, center_x + col_pitch]
}

pub fn draw_faction_selection_screen(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    delta_time: f32,
    mouse_position: Vector2,
) {
    let half_width = SCREEN_WIDTH / 2;
    d.draw_rectangle(0, 0, half_width, SCREEN_HEIGHT, Color::new(18, 18, 18, 255));
    d.draw_rectangle(
        half_width,
        0,
        SCREEN_WIDTH - half_width,
        SCREEN_HEIGHT,
        Color::new(18, 18, 18, 255),
    );

    // surbrillance du côté survolé par la souris
    let hovered_x = if mouse_position.x < half_width as f32 {
        0
    } else {
        half_width
    };
    d.draw_rectangle(
        hovered_x,
        0,
        half_width,
        SCREEN_HEIGHT,
        Color::new(255, 255, 255, 5),
    );

    let title = "Choose your faction";
    let title_size = 64.0;
    let title_text_size = assets.hud_font.measure_text(title, title_size, 1.0);
    d.draw_text_ex(
        &assets.hud_font,
        title,
        Vector2::new(SCREEN_WIDTH as f32 / 2.0 - title_text_size.x / 2.0, 30.0),
        title_size,
        1.0,
        Color::WHITE,
    );

    let human_center_x = SCREEN_WIDTH as f32 / 4.0;
    let undead_center_x = SCREEN_WIDTH as f32 * 3.0 / 4.0;

    for (label, center_x) in [("Human", human_center_x), ("Undead", undead_center_x)] {
        let label_size = 48.0;
        let label_text_size = assets.hud_font.measure_text(label, label_size, 1.0);
        d.draw_text_ex(
            &assets.hud_font,
            label,
            Vector2::new(center_x - label_text_size.x / 2.0, 130.0),
            label_size,
            1.0,
            Color::WHITE,
        );
    }

    let sprite_size = 250.0;
    let col_pitch = 170.0;
    let row_gap = 10.0;
    let rows_top = 240.0;
    let rows_bottom = SCREEN_HEIGHT as f32;
    let rows_height = sprite_size * 2.0 + row_gap;
    let row1_y = rows_top + (rows_bottom - rows_top - rows_height) / 2.0;
    let row2_y = row1_y + sprite_size + row_gap;

    let human_cols = faction_column_centers(human_center_x, col_pitch);
    let undead_cols = faction_column_centers(undead_center_x, col_pitch);

    // Human : soldier / cavalry / assassin en haut, longbowman / mage / priest en bas
    draw_idle_card(
        d,
        &mut assets.soldier.idle,
        human_cols[0],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.cavalry.idle,
        human_cols[1],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.assassin.idle,
        human_cols[2],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.longbowman.idle,
        human_cols[0],
        row2_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.mage.idle,
        human_cols[1],
        row2_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.priest.idle,
        human_cols[2],
        row2_y,
        sprite_size,
        delta_time,
    );

    // Undead : blood knight / banshee / wraith en haut, ghoul / necromancer / skeleton en bas
    draw_idle_card(
        d,
        &mut assets.blood_knight.idle,
        undead_cols[0],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.banshee.idle,
        undead_cols[1],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.wraith.idle,
        undead_cols[2],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.ghoul.idle,
        undead_cols[0],
        row2_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.necromancer.idle,
        undead_cols[1],
        row2_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.skeleton.idle,
        undead_cols[2],
        row2_y,
        sprite_size,
        delta_time,
    );

    d.draw_texture_ex(
        assets.cursor_texture(CursorType::Normal),
        Vector2::new(mouse_position.x, mouse_position.y),
        0.0,
        0.7,
        Color::WHITE,
    );
}

// Grid screen mode ------------------------------------------------------------
pub fn draw_grid_screen(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    delta_time: f32,
    tile_map: &TileMap,
    hud: &HudRects,
    units: &[Unit],
    game_mode: GameMode,
    current_turn: &TurnPhase,
    wait_button_visible: bool,
    move_range: &[(i32, i32)],
    valid_attack_positions: &[(i32, i32)],
    attackable_enemy_positions: &[(i32, i32)],
    healable_ally_positions: &[(i32, i32)],
    cursor_grid_x: i32,
    cursor_grid_y: i32,
    cursor_type: CursorType,
    mouse_position: Vector2,
    selected_unit: Option<&Unit>,
    inspected_enemy: Option<&Unit>,
    grid_speed: f32,
    duel_preview: Option<&DuelPreview>,
    position_scores: &[((i32, i32), i32)],
) {
    d.clear_background(Color::BEIGE);
    tile_map.draw(d);

    // for i in (0..SCREEN_HEIGHT).step_by(TILE_SIZE as usize) {
    //     d.draw_rectangle_lines(0, i, SCREEN_WIDTH, 1, Color::BLACK);
    // }
    //
    // for i in (0..SCREEN_WIDTH).step_by(TILE_SIZE as usize) {
    //     d.draw_rectangle_lines(i, 0, 1, SCREEN_HEIGHT, Color::BLACK);
    // }

    match current_turn {
        TurnPhase::PlayerTurn => {
            draw_texture_at(d, &assets.banner_your_turn, hud.banner_your_turn);
            draw_texture_at(d, &assets.btn_end_turn, hud.btn_end_turn);
            if wait_button_visible {
                draw_texture_at(d, &assets.btn_wait, hud.btn_wait);
            }
        }
        TurnPhase::EnemyTurn => {
            draw_texture_at(d, &assets.banner_enemy_turn, hud.banner_enemy_turn);
        }
    }
    if game_mode == GameMode::GridScreen {
        let label = if grid_speed > 1.0 { "x2" } else { "x1" };
        ui::draw_text_button(d, &assets.hud_font, hud.btn_speed, label);
        if grid_speed > 1.0 {
            // contour doré quand l'accéléré est actif
            d.draw_rectangle_lines_ex(hud.btn_speed, 3.0, Color::GOLD);
        }
    }

    for (x, y) in move_range {
        d.draw_rectangle(
            x * TILE_SIZE,
            y * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
            Color::new(0, 100, 255, 100), // bleu transparent
        );
    }

    if units.iter().any(|u| u.state == UnitState::ChoosingPosition) {
        for (x, y) in valid_attack_positions {
            d.draw_rectangle(
                x * TILE_SIZE,
                y * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                Color::new(255, 255, 0, 170), // jaune transparent
            );
        }
    }

    for (x, y) in attackable_enemy_positions {
        d.draw_rectangle(
            x * TILE_SIZE,
            y * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
            Color::new(255, 0, 0, 180), // rouge transparent
        );
    }

    for (x, y) in healable_ally_positions {
        d.draw_rectangle(
            x * TILE_SIZE,
            y * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
            Color::new(0, 220, 0, 160), // vert transparent
        );
    }

    if move_range.contains(&(cursor_grid_x, cursor_grid_y)) {
        d.draw_texture_ex(
            &assets.mouse_select_texture,
            Vector2::new(
                (cursor_grid_x * TILE_SIZE) as f32,
                (cursor_grid_y * TILE_SIZE) as f32,
            ),
            0.0,
            1.0,
            Color::WHITE,
        );
    }

    for unit in units {
        if unit.is_alive() {
            let animation = assets
                .animation_set_mut(unit.class)
                .for_state_mut(unit.state);
            draw_unit_sprite(
                d,
                animation,
                unit,
                unit.screen_x,
                unit.screen_y,
                128.0,
                delta_time,
            );
        }
    }

    draw_position_scores(d, assets, position_scores);
    if let Some(preview) = duel_preview {
        draw_duel_preview(d, assets, units, preview);
    }

    if let Some(unit) = selected_unit {
        draw_unit_info(d, assets, unit, winfo::panel_rect());
    }
    if let Some(enemy) = inspected_enemy.filter(|u| u.is_alive()) {
        draw_unit_info(d, assets, enemy, winfo::enemy_panel_rect(hud));
    }

    // écran de fin ----------------------------------------------------------
    let end_text = match game_mode {
        GameMode::Victory => Some(("VICTORY", Color::GOLD)),
        GameMode::Defeat => Some(("DEFEAT", Color::RED)),
        _ => None,
    };
    if let Some((text, color)) = end_text {
        let font_size = 65.0;
        let text_size = assets.hud_font.measure_text(text, font_size, 1.0);
        d.draw_text_ex(
            &assets.hud_font,
            text,
            Vector2::new(
                SCREEN_WIDTH as f32 / 2.0 - text_size.x / 2.0,
                SCREEN_HEIGHT as f32 / 2.0 - text_size.y / 2.0,
            ),
            font_size,
            1.0,
            color,
        );
        for (texture, rect) in [
            (&assets.btn_retry, hud.btn_retry),
            (&assets.btn_back, hud.btn_back),
            (&assets.btn_exit, hud.btn_exit),
        ] {
            draw_texture_at(d, texture, rect);
        }
    }

    d.draw_texture_ex(
        assets.cursor_texture(cursor_type),
        Vector2::new(mouse_position.x, mouse_position.y),
        0.0,
        0.7,
        Color::WHITE,
    );
}

/// petite fenêtre de stats à côté du perso sélectionné
fn draw_unit_info(d: &mut RaylibDrawHandle, assets: &Assets, unit: &Unit, rect: Rectangle) {
    d.draw_rectangle_rec(rect, Color::new(30, 30, 30, 220));
    let s = winfo::SCALE;
    d.draw_rectangle_lines_ex(rect, 2.0 * s, Color::WHITE);

    let pad = 6.0 * s;
    let x = rect.x + pad;
    let mut y = rect.y + pad;

    d.draw_text_ex(
        &assets.info_font,
        &unit.name,
        Vector2::new(x, y),
        16.0 * s,
        1.0,
        Color::WHITE,
    );
    y += 18.0 * s;

    ui::draw_health_bar(
        d,
        x,
        y,
        winfo::WIDTH - 2.0 * pad,
        6.0 * s,
        unit.hp_points,
        unit.hp_max_points,
    );
    y += 10.0 * s;

    let lines = [
        format!("HP {}/{}", unit.hp_points, unit.hp_max_points),
        format!("ATK {}  DEF {}", unit.attack_power, unit.defense),
        format!(
            "RNG {}  MOV {}/{}",
            unit.attack_range, unit.move_points_remaining, unit.move_points
        ),
    ];
    for line in &lines {
        d.draw_text_ex(
            &assets.info_font,
            line,
            Vector2::new(x, y),
            14.0 * s,
            1.0,
            Color::WHITE,
        );
        y += 15.0 * s;
    }
}

// écran de combat ------------------------------------------------------------
pub fn draw_combat_screen(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    delta_time: f32,
    attacker: &Unit,
    defender: &Unit,
    attacker_combat_x: f32,
    defender_combat_x: f32,
    minigame: Option<&MiniGame>,
    result_display: Option<&(TimingResult, f32, f32)>,
    conditions: &DuelConditions,
    player_is_attacker: bool,
) {
    draw_fullscreen_texture(d, &assets.combat_screen_background_texture);

    // HUD combat ---------------------------------------------------------------
    draw_unit_hud(d, assets, attacker);
    draw_unit_hud(d, assets, defender);

    // sprites ------------------------------------------------------------------
    let sprite_size = 500.0;
    let combat_y = SCREEN_HEIGHT as f32 / 2.0 - sprite_size / 2.0;

    if attacker.is_alive() {
        let animation = if attacker.state == UnitState::Healing {
            assets.heal_animation_mut(attacker.class).0
        } else {
            assets
                .animation_set_mut(attacker.class)
                .for_state_mut(attacker.state)
        };
        draw_unit_sprite(
            d,
            animation,
            attacker,
            attacker_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    }
    if defender.is_alive() {
        let animation = assets
            .animation_set_mut(defender.class)
            .for_state_mut(defender.state);
        draw_unit_sprite(
            d,
            animation,
            defender,
            defender_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    }
    // les effets d'attaque/soin se dessinent après les deux sprites pour rester au premier plan
    if attacker.state == UnitState::Attacking {
        let effect = &mut assets.animation_set_mut(attacker.class).attack_effect;
        draw_unit_sprite(
            d,
            effect,
            attacker,
            attacker_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    } else if attacker.state == UnitState::Healing {
        let effect = assets.heal_animation_mut(attacker.class).1;
        draw_unit_sprite(
            d,
            effect,
            attacker,
            attacker_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    }
    if defender.state == UnitState::Attacking {
        let effect = &mut assets.animation_set_mut(defender.class).attack_effect;
        draw_unit_sprite(
            d,
            effect,
            defender,
            defender_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    }

    // MiniGame -----------------------------------------------------------------
    if let Some(minigame) = minigame {
        minigame.draw(d, assets);
        draw_combat_factors(d, assets, conditions, player_is_attacker);
    }
    if let Some((result, _, multiplier)) = result_display {
        // _ c'est le "time_left" qu'on a pas besoin d'utiliser ici
        draw_timing_result(d, assets, *result, *multiplier);
    }
}

/// barre de vie + nom gauche droite en fonction de ou il regarde
fn draw_unit_hud(d: &mut RaylibDrawHandle, assets: &Assets, unit: &Unit) {
    let bar_width = 300.0;
    let bar_height = 30.0;
    let padding = 50.0;
    let bar_y = padding;

    let bar_x = if unit.facing_left {
        SCREEN_WIDTH as f32 - padding - bar_width
    } else {
        padding
    };

    ui::draw_health_bar(
        d,
        bar_x,
        bar_y,
        bar_width,
        bar_height,
        unit.hp_points,
        unit.hp_max_points,
    );
    d.draw_text_ex(
        &assets.hud_font,
        &unit.name,
        Vector2::new(bar_x, bar_y + bar_height + 10.0),
        24.0,
        1.0,
        Color::BLACK,
    );
}

fn draw_timing_result(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    result: TimingResult,
    multiplier: f32,
) {
    let (text, color) = match result {
        TimingResult::Bad => ("BAD", Color::RED),
        TimingResult::Good => ("GOOD", Color::ORANGE),
        TimingResult::Perfect => ("PERFECT", Color::LIME),
    };
    let full_text = format!("{} x{:.1}", text, multiplier);

    let text_size = assets.hud_font.measure_text(&full_text, 40.0, 1.0);
    d.draw_text_ex(
        &assets.alert_font,
        &full_text,
        Vector2::new(
            SCREEN_WIDTH as f32 / 2.0 - text_size.x / 2.0,
            SCREEN_HEIGHT as f32 / 2.0 - text_size.y / 2.0 - 350.0,
        ),
        40.0,
        1.0,
        color,
    );
}

// preview du duel ------------------------------------------------------------

const GOOD_FOR_ME: Color = Color::new(120, 230, 120, 255);
const BAD_FOR_ME: Color = Color::new(255, 110, 110, 255);
const MUTED: Color = Color::new(180, 180, 180, 255);
// taille globale des fenêtres de preview (panneau, badges, facteurs en combat)
const PREVIEW_SCALE: f32 = 1.6;

fn score_color(score: i32) -> Color {
    match score {
        s if s > 0 => GOOD_FOR_ME,
        s if s < 0 => BAD_FOR_ME,
        _ => Color::WHITE,
    }
}

/// petit badge +2 / -1 dans le coin de chaque case d'attaque possible
fn draw_position_scores(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    position_scores: &[((i32, i32), i32)],
) {
    for ((x, y), score) in position_scores {
        let text = if *score > 0 {
            format!("+{}", score)
        } else {
            score.to_string()
        };
        // "+12" doit tenir dans une case de 48 px
        let font_size = 12.0 * PREVIEW_SCALE;
        let size = assets.info_font.measure_text(&text, font_size, 1.0);
        let rect = Rectangle::new(
            (x * TILE_SIZE) as f32 + 2.0,
            (y * TILE_SIZE) as f32 + 2.0,
            size.x + 6.0 * PREVIEW_SCALE,
            size.y + 2.0 * PREVIEW_SCALE,
        );
        d.draw_rectangle_rec(rect, Color::new(20, 20, 20, 200));
        d.draw_text_ex(
            &assets.info_font,
            &text,
            Vector2::new(rect.x + 3.0 * PREVIEW_SCALE, rect.y + PREVIEW_SCALE),
            font_size,
            1.0,
            score_color(*score),
        );
    }
}

/// mini barre de timing : zone jaune + parfait vert, à l'échelle de la vraie
fn draw_mini_bar(d: &mut RaylibDrawHandle, x: f32, y: f32, width: f32, side: &DuelSide) {
    let h = (8.0 * PREVIEW_SCALE) as i32;
    d.draw_rectangle(x as i32, y as i32, width as i32, h, Color::DARKGRAY);
    let zone_x = x + width / 2.0 - side.window * width / 2.0;
    d.draw_rectangle(
        zone_x as i32,
        y as i32,
        (side.window * width) as i32,
        h,
        Color::YELLOW,
    );
    let perfect_x = x + width / 2.0 - side.precision * width / 2.0;
    d.draw_rectangle(
        perfect_x as i32,
        y as i32,
        (side.precision * width).ceil() as i32,
        h,
        Color::GREEN,
    );
}

fn side_summary(side: &DuelSide) -> String {
    format!(
        "window {}%  precision {}%  tempo x{:.1}  lives {}",
        (side.window * 100.0).round(),
        (side.precision * 100.0).round(),
        side.tempo,
        side.lives
    )
}

/// panneau en haut à gauche : dégâts prévus, barres des deux camps, facteurs de placement
fn draw_duel_preview(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    units: &[Unit],
    preview: &DuelPreview,
) {
    let attacker = &units[preview.attacker_idx];
    let defender = &units[preview.defender_idx];
    let c = &preview.conditions;

    // marque la case d'où partirait l'attaque si on doit bouger
    if preview.from != (attacker.grid_x, attacker.grid_y) {
        d.draw_rectangle_lines_ex(
            Rectangle::new(
                (preview.from.0 * TILE_SIZE) as f32,
                (preview.from.1 * TILE_SIZE) as f32,
                TILE_SIZE as f32,
                TILE_SIZE as f32,
            ),
            3.0,
            Color::WHITE,
        );
    }

    let damage = |mult: f32| (c.base_damage as f32 * mult) as i32;
    let (bad, good, perfect) = (damage(0.5), damage(1.0), damage(1.5));
    let hp = defender.hp_points;
    let (ko_text, ko_color) = if hp - bad <= 0 {
        ("sure KO", Color::GOLD)
    } else if hp - good <= 0 {
        ("KO on GOOD", Color::GOLD)
    } else if hp - perfect <= 0 {
        ("KO on PERFECT", Color::ORANGE)
    } else {
        ("no KO", MUTED)
    };

    let mut lines: Vec<(String, Color)> = vec![
        (
            format!(
                "{} -> {}{}",
                attacker.name,
                defender.name,
                if preview.from_best_tile {
                    "  (best tile)"
                } else {
                    ""
                }
            ),
            Color::WHITE,
        ),
        (
            format!("DMG  bad {}  good {}  perfect {}", bad, good, perfect),
            Color::WHITE,
        ),
        (
            format!(
                "{} HP {} -> {} (good)   {}",
                defender.name,
                hp,
                (hp - good).max(0),
                ko_text
            ),
            ko_color,
        ),
    ];
    // chances estimées (même calcul que l'IA) et avantage du placement
    let odds = c.attacker.odds();
    let expected = (c.base_damage as f32 * odds.attack_multiplier()).round() as i32;
    let advantage = c.advantage();
    lines.push((
        format!(
            "Expected ~{} dmg   odds: perfect {}%  good {}%  bad {}%",
            expected,
            (odds.perfect * 100.0).round(),
            (odds.good * 100.0).round(),
            (odds.bad * 100.0).round()
        ),
        Color::WHITE,
    ));
    lines.push((
        format!(
            "Placement: {}{}% damage vs no modifiers",
            if advantage > 0 { "+" } else { "" },
            advantage
        ),
        score_color(advantage),
    ));
    let bars_y_index = lines.len();
    lines.push((
        format!("You    {}", side_summary(&c.attacker)),
        Color::WHITE,
    ));
    lines.push((format!("Them   {}", side_summary(&c.defender)), MUTED));
    if c.factors.is_empty() {
        lines.push((String::from("No positional modifiers"), MUTED));
    }
    for factor in &c.factors {
        let (sign, color) = if factor.favors_attacker {
            ("+ ", GOOD_FOR_ME)
        } else {
            ("- ", BAD_FOR_ME)
        };
        lines.push((format!("{}{}", sign, factor.label), color));
    }

    let font_size = 15.0 * PREVIEW_SCALE;
    let line_height = 17.0 * PREVIEW_SCALE;
    let mini_bar_height = 11.0 * PREVIEW_SCALE;
    let mini_bar_width = 160.0 * PREVIEW_SCALE;
    let pad = 8.0 * PREVIEW_SCALE;
    // largeur calée sur la ligne la plus longue
    let widest_line = lines
        .iter()
        .map(|(text, _)| assets.info_font.measure_text(text, font_size, 1.0).x)
        .fold(mini_bar_width, f32::max);
    let width = widest_line + pad * 2.0;
    let height = pad * 2.0 + lines.len() as f32 * line_height + 2.0 * mini_bar_height;
    // côté opposé à la cible pour pas cacher le combat, sous la bannière de tour à droite
    let panel = if defender.grid_x * TILE_SIZE < SCREEN_WIDTH / 2 {
        Rectangle::new(SCREEN_WIDTH as f32 - width - 10.0, 84.0, width, height)
    } else {
        Rectangle::new(10.0, 10.0, width, height)
    };
    d.draw_rectangle_rec(panel, Color::new(20, 20, 20, 225));
    d.draw_rectangle_lines_ex(panel, 2.0 * PREVIEW_SCALE, score_color(advantage));

    let x = panel.x + pad;
    let mut y = panel.y + pad;
    for (i, (text, color)) in lines.iter().enumerate() {
        d.draw_text_ex(
            &assets.info_font,
            text,
            Vector2::new(x, y),
            font_size,
            1.0,
            *color,
        );
        y += line_height;
        // une mini barre sous chaque ligne "You" / "Them"
        if i == bars_y_index || i == bars_y_index + 1 {
            let side = if i == bars_y_index {
                &c.attacker
            } else {
                &c.defender
            };
            draw_mini_bar(d, x, y, mini_bar_width, side);
            y += mini_bar_height;
        }
    }
}

// combat : rappel des facteurs ------------------------------------------

fn draw_combat_factors(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    conditions: &DuelConditions,
    player_is_attacker: bool,
) {
    let font_size = 16.0 * PREVIEW_SCALE;
    let line_height = 18.0 * PREVIEW_SCALE;
    let mut y = SCREEN_HEIGHT as f32 - 20.0 - conditions.factors.len() as f32 * line_height;
    for factor in &conditions.factors {
        let good_for_me = factor.favors_attacker == player_is_attacker;
        let (sign, color) = if good_for_me {
            ("+ ", GOOD_FOR_ME)
        } else {
            ("- ", BAD_FOR_ME)
        };
        let text = format!("{}{}", sign, factor.label);
        let size = assets.info_font.measure_text(&text, font_size, 1.0);
        let rect = Rectangle::new(16.0, y - 1.0, size.x + 8.0 * PREVIEW_SCALE, line_height);
        d.draw_rectangle_rec(rect, Color::new(20, 20, 20, 190));
        d.draw_text_ex(
            &assets.info_font,
            &text,
            Vector2::new(16.0 + 4.0 * PREVIEW_SCALE, y),
            font_size,
            1.0,
            color,
        );
        y += line_height;
    }
}

// illustrations des mini-jeux dans le guide ------------------------------------

/// vue figée d'un mini-jeu dans un cadre, mêmes couleurs que le vrai jeu
fn draw_guide_illustration(d: &mut impl RaylibDraw, kind: &guide::Illustration, rect: Rectangle) {
    d.draw_rectangle_rec(rect, Color::new(10, 10, 10, 220));
    d.draw_rectangle_lines_ex(rect, 2.0, Color::new(255, 255, 255, 90));
    let center = Vector2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);

    match kind {
        guide::Illustration::Target => {
            // trajectoire en 8 en pointillés, la cible dessus, le viseur un peu à côté
            for i in 0..48 {
                let t = i as f32 / 48.0 * std::f32::consts::TAU;
                let p = Vector2::new(
                    center.x + 110.0 * (1.6 * t).sin(),
                    center.y + 30.0 * (2.3 * t + 1.0).sin(),
                );
                d.draw_circle_v(p, 1.5, Color::new(255, 255, 255, 70));
            }
            let target = Vector2::new(center.x + 50.0, center.y - 5.0);
            d.draw_circle_v(target, 22.0, Color::YELLOW);
            d.draw_circle_v(target, 6.0, Color::GREEN);
            let aim = Vector2::new(center.x + 38.0, center.y + 6.0);
            d.draw_ring(aim, 8.0, 10.0, 0.0, 360.0, 24, Color::WHITE);
            for dir in [
                Vector2::new(1.0, 0.0),
                Vector2::new(-1.0, 0.0),
                Vector2::new(0.0, 1.0),
                Vector2::new(0.0, -1.0),
            ] {
                d.draw_line_ex(aim + dir * 4.0, aim + dir * 15.0, 2.0, Color::WHITE);
            }
        }
        guide::Illustration::Charge => {
            d.draw_ring(center, 22.0, 32.0, 0.0, 360.0, 48, Color::YELLOW);
            d.draw_ring(center, 26.0, 28.0, 0.0, 360.0, 48, Color::GREEN);
            d.draw_circle_v(center, 4.0, Color::WHITE);
            d.draw_ring(center, 41.0, 44.0, 0.0, 360.0, 48, Color::WHITE);
            // flèches vers le centre : l'anneau se resserre
            for side in [-1.0, 1.0] {
                let from = Vector2::new(center.x + side * 95.0, center.y);
                let to = Vector2::new(center.x + side * 58.0, center.y);
                draw_guide_arrow(d, from, to, Color::new(255, 255, 255, 160));
            }
        }
        guide::Illustration::Sequence => {
            let size = 46.0;
            let gap = 10.0;
            let total = size * 4.0 + gap * 3.0;
            let start_x = center.x - total / 2.0;
            let top = rect.y + 14.0;
            let arrows = [
                Vector2::new(0.0, -1.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(0.0, -1.0),
                Vector2::new(-1.0, 0.0),
            ];
            for (i, dir) in arrows.iter().enumerate() {
                let r = Rectangle::new(start_x + i as f32 * (size + gap), top, size, size);
                let (background, border) = match i {
                    0 | 1 => (Color::new(40, 120, 40, 230), Color::GREEN),
                    2 => (Color::new(30, 30, 30, 230), Color::WHITE),
                    _ => (Color::new(30, 30, 30, 160), Color::new(255, 255, 255, 90)),
                };
                d.draw_rectangle_rec(r, background);
                d.draw_rectangle_lines_ex(r, 2.0, border);
                let c = Vector2::new(r.x + size / 2.0, r.y + size / 2.0);
                draw_guide_arrow(d, c - *dir * 13.0, c + *dir * 13.0, Color::WHITE);
            }
            draw_guide_time_bar(
                d,
                Rectangle::new(start_x, top + size + 10.0, total, 10.0),
                0.4,
                0.55,
            );
        }
        guide::Illustration::Runes => {
            let colors = [
                Color::new(170, 100, 255, 255),
                Color::new(60, 200, 255, 255),
                Color::new(255, 150, 40, 255),
                Color::new(255, 90, 170, 255),
            ];
            let radius = 20.0;
            let step = 62.0;
            let start_x = center.x - step * 1.5;
            let row_y = rect.y + 42.0;
            for (i, color) in colors.iter().enumerate() {
                let c = Vector2::new(start_x + i as f32 * step, row_y);
                // la 2e rune est « allumée » pendant la démo
                let lit = i == 1;
                let fill = if lit {
                    *color
                } else {
                    Color::new(color.r / 3, color.g / 3, color.b / 3, 255)
                };
                let r = if lit { radius * 1.15 } else { radius };
                d.draw_poly(c, 3 + i as i32, r, -90.0, fill);
                d.draw_poly_lines_ex(
                    c,
                    3 + i as i32,
                    r,
                    -90.0,
                    2.0,
                    Color::new(255, 255, 255, 120),
                );
            }
            for i in 0..4 {
                let p = Vector2::new(center.x + (i as f32 - 1.5) * 12.0, rect.y + 12.0);
                if i < 1 {
                    d.draw_circle_v(p, 4.0, Color::WHITE);
                } else {
                    d.draw_circle_lines(p.x as i32, p.y as i32, 4.0, Color::WHITE);
                }
            }
            draw_guide_time_bar(
                d,
                Rectangle::new(
                    start_x - radius,
                    rect.y + 74.0,
                    step * 3.0 + radius * 2.0,
                    10.0,
                ),
                0.4,
                0.25,
            );
        }
        guide::Illustration::Channel => {
            // aura qui dérive (traînée), curseur dans le cœur, jauge de concentration
            for i in 0..5 {
                let p = Vector2::new(
                    center.x - 90.0 + i as f32 * 22.0,
                    center.y - 12.0 + (i as f32 * 0.9).sin() * 10.0,
                );
                d.draw_circle_v(p, 4.0, Color::new(255, 240, 150, 40 + i as u8 * 20));
            }
            let aura = Vector2::new(center.x + 40.0, center.y - 10.0);
            d.draw_circle_v(aura, 34.0, Color::new(255, 240, 150, 80));
            d.draw_circle_v(aura, 26.0, Color::YELLOW);
            d.draw_circle_v(aura, 10.0, Color::GREEN);
            let aim = Vector2::new(aura.x + 4.0, aura.y + 3.0);
            d.draw_circle_v(aim, 5.0, Color::WHITE);
            d.draw_circle_lines(aim.x as i32, aim.y as i32, 5.0, Color::BLACK);
            let meter = Rectangle::new(
                rect.x + 20.0,
                rect.y + rect.height - 20.0,
                rect.width - 40.0,
                10.0,
            );
            d.draw_rectangle_rec(meter, Color::new(60, 60, 60, 230));
            d.draw_rectangle(
                (meter.x + meter.width * 0.5) as i32,
                meter.y as i32,
                (meter.width * 0.35) as i32,
                meter.height as i32,
                Color::new(255, 255, 0, 110),
            );
            d.draw_rectangle(
                (meter.x + meter.width * 0.85) as i32,
                meter.y as i32,
                (meter.width * 0.15) as i32,
                meter.height as i32,
                Color::new(0, 228, 48, 110),
            );
            d.draw_rectangle(
                meter.x as i32,
                meter.y as i32 + 2,
                (meter.width * 0.62) as i32,
                meter.height as i32 - 4,
                Color::WHITE,
            );
        }
        guide::Illustration::Heartbeat => {
            let track = Rectangle::new(rect.x + 14.0, rect.y + 58.0, rect.width - 28.0, 28.0);
            let line_x = track.x + 50.0;
            // cœur au-dessus de la ligne de frappe
            let heart = Vector2::new(line_x, rect.y + 26.0);
            let r = 9.0;
            let red = Color::new(200, 20, 40, 255);
            d.draw_circle_v(Vector2::new(heart.x - r * 0.9, heart.y - r * 0.3), r, red);
            d.draw_circle_v(Vector2::new(heart.x + r * 0.9, heart.y - r * 0.3), r, red);
            let a = Vector2::new(heart.x - r * 1.85, heart.y);
            let b = Vector2::new(heart.x + r * 1.85, heart.y);
            let tip = Vector2::new(heart.x, heart.y + r * 2.0);
            d.draw_triangle(a, tip, b, red);
            d.draw_triangle(a, b, tip, red);

            d.draw_rectangle_rec(track, Color::new(50, 15, 20, 230));
            d.draw_rectangle(
                (line_x - 14.0) as i32,
                track.y as i32,
                28,
                track.height as i32,
                Color::YELLOW,
            );
            d.draw_rectangle(
                (line_x - 6.0) as i32,
                track.y as i32,
                12,
                track.height as i32,
                Color::GREEN,
            );
            d.draw_rectangle(
                line_x as i32 - 1,
                track.y as i32 - 4,
                3,
                track.height as i32 + 8,
                Color::WHITE,
            );
            let drop_y = track.y + track.height / 2.0;
            for drop_x in [line_x + 70.0, line_x + 145.0, line_x + 220.0] {
                d.draw_circle_v(
                    Vector2::new(drop_x, drop_y),
                    9.0,
                    Color::new(220, 30, 50, 255),
                );
                d.draw_circle_lines(drop_x as i32, drop_y as i32, 9.0, Color::WHITE);
            }
            draw_guide_arrow(
                d,
                Vector2::new(track.x + track.width - 20.0, rect.y + 26.0),
                Vector2::new(track.x + track.width - 80.0, rect.y + 26.0),
                Color::new(255, 255, 255, 160),
            );
        }
    }
}

/// barre de temps des jeux « finir avant la fin » : vert = PERFECT, jaune = GOOD
fn draw_guide_time_bar(d: &mut impl RaylibDraw, bar: Rectangle, perfect_share: f32, progress: f32) {
    d.draw_rectangle_rec(bar, Color::YELLOW);
    d.draw_rectangle(
        bar.x as i32,
        bar.y as i32,
        (bar.width * perfect_share) as i32,
        bar.height as i32,
        Color::GREEN,
    );
    d.draw_rectangle(
        (bar.x + bar.width * progress) as i32 - 2,
        bar.y as i32 - 3,
        3,
        bar.height as i32 + 6,
        Color::WHITE,
    );
}

/// flèche simple de `from` vers `to` (trait + pointe)
fn draw_guide_arrow(d: &mut impl RaylibDraw, from: Vector2, to: Vector2, color: Color) {
    let length = from.distance(to);
    if length <= 0.0 {
        return;
    }
    let dir = (to - from) * (1.0 / length);
    let side = Vector2::new(-dir.y, dir.x);
    let head = (length * 0.45).min(10.0);
    let base = to - dir * head;
    d.draw_line_ex(from, base, 3.0, color);
    let left = base + side * (head * 0.8);
    let right = base - side * (head * 0.8);
    // raylib ne dessine un triangle que dans le sens antihoraire : on essaie les deux
    d.draw_triangle(to, left, right, color);
    d.draw_triangle(to, right, left, color);
}
