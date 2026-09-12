use crate::animation::Animation;
use crate::assets::Assets;
use crate::cursor::CursorType;
use crate::game_mode::{GameMode, TurnPhase};
use crate::map::TileMap;
use crate::minigame::{TimingBar, TimingResult};
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

    for background in [
        &assets.title_screen_background_texture_1,
        &assets.title_screen_background_texture_2,
        &assets.title_screen_background_texture_3,
        &assets.title_screen_background_texture_4,
        &assets.title_screen_background_texture_5,
    ] {
        draw_fullscreen_texture(d, background);
    }

    draw_texture_at(d, &assets.title, hud.title);
    draw_texture_at(d, &assets.btn_play_title_screen, hud.btn_play_title_screen);
    draw_texture_at(
        d,
        &assets.btn_settings_title_screen,
        hud.btn_settings_title_screen,
    );
    draw_texture_at(d, &assets.btn_exit_title_screen, hud.btn_exit_title_screen);

    d.draw_texture_ex(
        assets.cursor_texture(CursorType::Normal),
        Vector2::new(mouse_position.x, mouse_position.y),
        0.0,
        0.7,
        Color::WHITE,
    );
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
    thread: &RaylibThread,
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
    // raylib "batch" les draw calls (le vrai rendu GPU arrive au flush du batch, pas
    // ici) alors que set_texture_filter change l'état de la texture immédiatement :
    // remettre "nearest" juste après ce draw_texture_pro annulerait le bilinéaire
    // avant même que ce dessin soit vraiment envoyé au GPU. On laisse donc en
    // bilinéaire ; draw_grid_screen/draw_combat_screen remettent "nearest" eux-mêmes
    // avant de dessiner, pour ne pas hériter de cet état.
    animation
        .texture
        .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
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

/// centres des 3 colonnes de sprites pour une faction, centrées sur `center_x`
fn faction_column_centers(center_x: f32, sprite_size: f32, col_gap: f32) -> [f32; 3] {
    let total_width = sprite_size * 3.0 + col_gap * 2.0;
    let first = center_x - total_width / 2.0 + sprite_size / 2.0;
    [
        first,
        first + sprite_size + col_gap,
        first + 2.0 * (sprite_size + col_gap),
    ]
}

pub fn draw_faction_selection_screen(
    d: &mut RaylibDrawHandle,
    thread: &RaylibThread,
    assets: &mut Assets,
    delta_time: f32,
    mouse_position: Vector2,
) {
    let half_width = SCREEN_WIDTH / 2;
    d.draw_rectangle(
        0,
        0,
        half_width,
        SCREEN_HEIGHT,
        Color::new(40, 70, 140, 255),
    );
    d.draw_rectangle(
        half_width,
        0,
        SCREEN_WIDTH - half_width,
        SCREEN_HEIGHT,
        Color::new(140, 30, 30, 255),
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
        Color::new(255, 255, 255, 15),
    );

    d.draw_rectangle(SCREEN_WIDTH / 2 - 2, 0, 4, SCREEN_HEIGHT, Color::WHITE);

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

    let sprite_size = 170.0;
    let col_gap = 16.0;
    let row_gap = 20.0;
    let rows_top = 240.0;
    let rows_bottom = SCREEN_HEIGHT as f32;
    let rows_height = sprite_size * 2.0 + row_gap;
    let row1_y = rows_top + (rows_bottom - rows_top - rows_height) / 2.0;
    let row2_y = row1_y + sprite_size + row_gap;

    let human_cols = faction_column_centers(human_center_x, sprite_size, col_gap);
    let undead_cols = faction_column_centers(undead_center_x, sprite_size, col_gap);

    // Human : soldier / cavalry / assassin en haut, longbowman / mage / priest en bas
    draw_idle_card(
        d,
        thread,
        &mut assets.soldier.idle,
        human_cols[0],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.cavalry.idle,
        human_cols[1],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.assassin.idle,
        human_cols[2],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.longbowman.idle,
        human_cols[0],
        row2_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.mage.idle,
        human_cols[1],
        row2_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.priest.idle,
        human_cols[2],
        row2_y,
        sprite_size,
        delta_time,
    );

    // Undead : blood knight / banshee / wraith en haut, ghoul / necromancer / skeleton en bas
    draw_idle_card(
        d,
        thread,
        &mut assets.blood_knight.idle,
        undead_cols[0],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.banshee.idle,
        undead_cols[1],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.wraith.idle,
        undead_cols[2],
        row1_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.ghoul.idle,
        undead_cols[0],
        row2_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
        &mut assets.necromancer.idle,
        undead_cols[1],
        row2_y,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        thread,
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

/// remet le filtrage "nearest" (pixel-art net) sur les textures idle des unités
/// jouables, au cas où l'écran de sélection de faction les aurait laissées en
/// bilinéaire (voir draw_idle_card)
fn reset_unit_texture_filters(assets: &Assets, thread: &RaylibThread) {
    for texture in [&assets.soldier.idle.texture, &assets.wraith.idle.texture] {
        texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_POINT);
    }
}

// Grid screen mode ------------------------------------------------------------
pub fn draw_grid_screen(
    d: &mut RaylibDrawHandle,
    thread: &RaylibThread,
    assets: &mut Assets,
    delta_time: f32,
    tile_map: &TileMap,
    hud: &HudRects,
    soldier: &Unit,
    wraith: &Unit,
    game_mode: GameMode,
    current_turn: &TurnPhase,
    wait_button_visible: bool,
    move_range: &[(i32, i32)],
    valid_attack_positions: &[(i32, i32)],
    wraith_attackable: bool,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
    cursor_type: CursorType,
    mouse_position: Vector2,
) {
    // l'écran de sélection de faction laisse ces textures en filtrage bilinéaire
    // (voir draw_idle_card) ; on remet "nearest" avant de dessiner le pixel-art
    reset_unit_texture_filters(assets, thread);

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

    for (x, y) in move_range {
        d.draw_rectangle(
            x * TILE_SIZE,
            y * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
            Color::new(0, 100, 255, 100), // bleu transparent
        );
    }

    if soldier.state == UnitState::ChoosingPosition {
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

    if wraith_attackable {
        d.draw_rectangle(
            wraith.grid_x * TILE_SIZE,
            wraith.grid_y * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
            Color::new(255, 0, 0, 180), // rouge transparent
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

    if soldier.is_alive() {
        let animation = assets.soldier.for_state_mut(soldier.state);
        draw_unit_sprite(
            d,
            animation,
            soldier,
            soldier.screen_x,
            soldier.screen_y,
            128.0,
            delta_time,
        );
    }

    if wraith.is_alive() {
        let animation = assets.wraith.for_state_mut(wraith.state);
        draw_unit_sprite(
            d,
            animation,
            wraith,
            wraith.screen_x,
            wraith.screen_y,
            128.0,
            delta_time,
        );
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

// écran de combat ------------------------------------------------------------
pub fn draw_combat_screen(
    d: &mut RaylibDrawHandle,
    thread: &RaylibThread,
    assets: &mut Assets,
    delta_time: f32,
    soldier: &Unit,
    wraith: &Unit,
    soldier_combat_x: f32,
    wraith_combat_x: f32,
    timing_bar: Option<&TimingBar>,
    result_display: Option<&(TimingResult, f32, f32)>,
) {
    // l'écran de sélection de faction laisse ces textures en filtrage bilinéaire
    // (voir draw_idle_card) ; on remet "nearest" avant de dessiner le pixel-art
    reset_unit_texture_filters(assets, thread);

    draw_fullscreen_texture(d, &assets.combat_screen_background_texture);

    // HUD combat ---------------------------------------------------------------
    draw_unit_hud(d, assets, soldier);
    draw_unit_hud(d, assets, wraith);

    // sprites ------------------------------------------------------------------
    let sprite_size = 500.0;
    let combat_y = SCREEN_HEIGHT as f32 / 2.0 - sprite_size / 2.0;

    if soldier.is_alive() {
        let animation = assets.soldier.for_state_mut(soldier.state);
        draw_unit_sprite(
            d,
            animation,
            soldier,
            soldier_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    }
    if soldier.state == UnitState::Attacking {
        let effect = &mut assets.soldier.attack_effect;
        draw_unit_sprite(
            d,
            effect,
            soldier,
            soldier_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    }
    if wraith.state == UnitState::Attacking {
        let effect = &mut assets.wraith.attack_effect;
        draw_unit_sprite(
            d,
            effect,
            wraith,
            wraith_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    }
    if wraith.is_alive() {
        let animation = assets.wraith.for_state_mut(wraith.state);
        draw_unit_sprite(
            d,
            animation,
            wraith,
            wraith_combat_x,
            combat_y,
            sprite_size,
            delta_time,
        );
    }

    // MiniGame -----------------------------------------------------------------
    if let Some(bar) = timing_bar {
        draw_timing_bar(d, bar);
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

fn draw_timing_bar(d: &mut RaylibDrawHandle, bar: &TimingBar) {
    let bar_x = SCREEN_WIDTH as f32 / 2.0 - 200.0;
    let bar_y = SCREEN_HEIGHT as f32 - 120.0;
    let bar_width = 400.0;
    let bar_height = 30.0;

    // fond de la barre
    d.draw_rectangle(
        bar_x as i32,
        bar_y as i32,
        bar_width as i32,
        bar_height as i32,
        Color::DARKGRAY,
    );

    // zone good
    let good_x = bar_x + bar.zone_start * bar_width;
    let good_width = (bar.zone_end - bar.zone_start) * bar_width;
    d.draw_rectangle(
        good_x as i32,
        bar_y as i32,
        good_width as i32,
        bar_height as i32,
        Color::YELLOW,
    );

    // zone perfect
    let perfect_x = bar_x + bar.perfect_zone_start * bar_width;
    let perfect_width = (bar.perfect_zone_end - bar.perfect_zone_start) * bar_width;
    d.draw_rectangle(
        perfect_x as i32,
        bar_y as i32,
        perfect_width as i32,
        bar_height as i32,
        Color::GREEN,
    );

    // curseur (ligne blanche qui bouge)
    let cursor_x = bar_x + bar.cursor_position * bar_width;
    d.draw_rectangle(
        cursor_x as i32 - 2,
        bar_y as i32 - 5,
        4,
        bar_height as i32 + 10,
        Color::WHITE,
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
