use crate::animation::Animation;
use crate::assets::Assets;
use std::net::IpAddr;

use crate::cursor::CursorType;
use crate::duel::{DuelConditions, DuelSide};
use crate::game_mode::{GameMode, TurnPhase};
use crate::grid::DuelPreview;
use crate::map::TileMap;
use crate::minigame::{MiniGame, MiniGameKind, ResultDisplay, TimingResult};
use crate::network::Browser;
use crate::screens::guide::{self, GuidePage};
use crate::screens::host::{self, HostField, HostForm};
use crate::screens::multiplayer;
use crate::screens::practice::{self, Cell, Phase as PracticePhase, Practice, Role};
use crate::screens::winfo;
use crate::settings::{self, Bind, tr};
use crate::trf;
use crate::ui::{self, HudRects, Skin};
use crate::unit::{Faction, Unit, UnitClass, UnitState};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, TILE_SIZE};
use raylib::prelude::*;

fn unit_source_rec(animation: &Animation, unit: &Unit) -> Rectangle {
    let mut source_rec = animation.animation_frame();
    if unit.facing_left {
        source_rec.width = -source_rec.width;
    }
    source_rec
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
    notice: Option<&str>,
    update_notice: Option<&str>,
) {
    d.clear_background(Color::BLACK);

    draw_fullscreen_texture(d, &assets.title_background_texture);

    if let Some(update_notice) = update_notice {
        let size = assets.info_font.measure_text(update_notice, 20.0, 1.0);
        d.draw_text_ex(
            &assets.info_font,
            update_notice,
            Vector2::new(
                SCREEN_WIDTH as f32 / 2.0 - size.x / 2.0,
                SCREEN_HEIGHT as f32 - size.y - 20.0,
            ),
            20.0,
            1.0,
            Color::GOLD,
        );
    }
    if let Some(notice) = notice {
        let text_size = assets.alert_font.measure_text(notice, 40.0, 1.0);
        d.draw_text_ex(
            &assets.alert_font,
            notice,
            // sous le dernier bouton du menu (Exit)
            Vector2::new(
                SCREEN_WIDTH as f32 / 2.0 - text_size.x / 2.0,
                hud.btn_exit_title_screen.y + hud.btn_exit_title_screen.height + 30.0,
            ),
            40.0,
            1.0,
            Color::RED,
        );
    }

    for (rect, label) in [
        (hud.btn_play_title_screen, tr("Solo", "Solo")),
        (hud.btn_multi_title_screen, tr("Multiplayer", "Multijoueur")),
        (hud.btn_settings_title_screen, tr("Settings", "Réglages")),
        (hud.btn_learn_title_screen, tr("Guide", "Guide")),
        (hud.btn_exit_title_screen, tr("Exit", "Quitter")),
    ] {
        ui::draw_button(d, assets, rect, label, Skin::Menu, mouse_position, true);
    }

    draw_cursor(d, assets, mouse_position);
}

// Pause screen --------------------------------------------------------------------
pub fn draw_pause_screen(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    hud: &HudRects,
    mouse_position: Vector2,
) {
    draw_fullscreen_texture(d, &assets.menu_background_texture);
    ui::draw_title(
        d,
        &assets.hud_font,
        tr("Pause", "Pause"),
        SCREEN_WIDTH as f32 / 2.0,
        hud.btn_pause_play.y - 48.0 - 30.0,
        48.0,
    );
    for (rect, label) in [
        (hud.btn_pause_play, tr("Resume", "Reprendre")),
        (hud.btn_pause_guide, tr("Guide", "Guide")),
        (hud.btn_pause_settings, tr("Settings", "Réglages")),
        (hud.btn_pause_back, tr("Main menu", "Menu")),
        (hud.btn_pause_exit, tr("Exit", "Quitter")),
    ] {
        ui::draw_button(d, assets, rect, label, Skin::Wood, mouse_position, true);
    }
    draw_cursor(d, assets, mouse_position);
}

/// curseur de la souris (le curseur système est caché)
fn draw_cursor(d: &mut RaylibDrawHandle, assets: &Assets, mouse_position: Vector2) {
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
    page: GuidePage,
    online: bool,
) {
    draw_fullscreen_texture(d, &assets.menu_background_texture);
    d.draw_rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::new(0, 0, 0, 80));

    let title = tr("How to play", "Comment jouer");
    let title_size = 46.0;
    ui::draw_title(d, &assets.hud_font, title, SCREEN_WIDTH as f32 / 2.0, 22.0, title_size);
    let _ = delta_time;
    let assets = &*assets;

    // onglets : les pages, puis l'entraînement
    let tabs = guide::tab_rects();
    for (i, tab) in tabs.iter().enumerate() {
        let (label, enabled) = match GuidePage::ALL.get(i) {
            Some(p) => (p.title().to_string(), true),
            None => (trf!("Training >" ; "Entraînement >"), !online),
        };
        ui::draw_button(d, assets, *tab, &label, Skin::Wood, mouse_position, enabled);
        if GuidePage::ALL.get(i) == Some(&page) {
            d.draw_rectangle_lines_ex(*tab, 3.0, ui::GOLD);
        }
    }

    match page {
        GuidePage::Basics => draw_guide_basics(d, assets),
        GuidePage::MiniGames => draw_guide_minigames(d, assets, mouse_position, online),
        GuidePage::Positioning => draw_guide_positioning(d, assets, mouse_position, online),
    }

    ui::draw_button(
        d,
        assets,
        hud.btn_guide_back,
        tr("Back", "Retour"),
        Skin::Wood,
        mouse_position,
        true,
    );
    draw_cursor(d, assets, mouse_position);
}

/// taille des cases des mini grilles du guide
const DIAGRAM_CELL: f32 = 34.0;
const TOKEN_BLUE: Color = Color::new(70, 130, 255, 255);
const TOKEN_RED: Color = Color::new(220, 60, 60, 255);

/// titre de carte (police pixel, couleur crème)
fn draw_card_title(d: &mut impl RaylibDraw, assets: &Assets, text: &str, pos: Vector2, size: f32) {
    d.draw_text_ex(&assets.alert_font, text, pos + Vector2::new(0.0, 2.0), size, 2.0, Color::new(0, 0, 0, 160));
    d.draw_text_ex(&assets.alert_font, text, pos, size, 2.0, ui::CREAM);
}

/// mini grille vide : cases d'herbe de `cell` px
fn draw_mini_grid(d: &mut RaylibDrawHandle, origin: Vector2, cols: i32, rows: i32, cell: f32) {
    for row in 0..rows {
        for col in 0..cols {
            let rect = Rectangle::new(origin.x + col as f32 * cell, origin.y + row as f32 * cell, cell, cell);
            d.draw_rectangle_rec(rect, Color::new(70, 95, 60, 255));
            d.draw_rectangle_lines_ex(rect, 1.0, Color::new(0, 0, 0, 90));
        }
    }
}

fn mini_cell(origin: Vector2, col: i32, row: i32, cell: f32) -> Rectangle {
    Rectangle::new(origin.x + col as f32 * cell, origin.y + row as f32 * cell, cell, cell)
}

/// jeton rond d'une unité dans une mini grille (anneau blanc = l'unité dont on parle)
fn draw_token(d: &mut impl RaylibDraw, cell: Rectangle, color: Color, ring: bool) {
    let center = Vector2::new(cell.x + cell.width / 2.0, cell.y + cell.height / 2.0);
    let radius = cell.width * 0.36;
    d.draw_circle_v(center, radius, color);
    if ring {
        d.draw_ring(center, radius, radius + 3.0, 0.0, 360.0, 24, Color::WHITE);
    }
}

fn draw_guide_basics(d: &mut RaylibDrawHandle, assets: &Assets) {
    let time = d.get_time() as f32;
    let steps = [
        (
            tr("Select", "Choisis"),
            tr("Click one of your units. Its stats show at the bottom.", "Clique sur une de tes unités. Ses stats s'affichent en bas."),
        ),
        (
            tr("Move", "Déplace"),
            tr("Blue tiles show where it can go. Click one to move.", "Les cases bleues montrent où aller. Clique pour t'y rendre."),
        ),
        (
            tr("Attack", "Attaque"),
            tr("Click a red enemy to attack. Healers click a green ally.", "Clique sur un ennemi rouge. Les soigneurs cliquent un allié vert."),
        ),
        (
            tr("Duel!", "Duel !"),
            tr("A mini-game sets the damage. Both sides play at once.", "Un mini-jeu décide des dégâts. Les deux camps jouent ensemble."),
        ),
    ];
    for (i, (title, text)) in steps.iter().enumerate() {
        let card = guide::step_rect(i);
        ui::draw_panel(d, card);
        // pastille numérotée
        let badge = Vector2::new(card.x + 30.0, card.y + 30.0);
        d.draw_circle_v(badge, 17.0, ui::GOLD);
        let number = (i + 1).to_string();
        let size = assets.alert_font.measure_text(&number, 28.0, 1.0);
        d.draw_text_ex(&assets.alert_font, &number, badge - size * 0.5 + Vector2::new(1.0, 1.0), 28.0, 1.0, Color::new(40, 28, 10, 255));
        draw_card_title(d, assets, &title.to_uppercase(), Vector2::new(card.x + 58.0, card.y + 16.0), 30.0);

        let scene = Rectangle::new(card.x + 16.0, card.y + 60.0, card.width - 32.0, 116.0);
        d.draw_rectangle_rec(scene, Color::new(8, 8, 10, 160));
        draw_basics_scene(d, assets, i, scene, time);

        ui::draw_wrapped(
            d,
            &assets.info_font,
            text,
            Vector2::new(card.x + 18.0, card.y + 188.0),
            card.width - 36.0,
            16.0,
            Color::new(225, 220, 210, 255),
        );
    }

    let (results, controls) = guide::basics_bottom_rects();
    // résultats du duel ----------------------------------------------------------
    ui::draw_panel(d, results);
    draw_card_title(d, assets, &tr("Duel results", "Résultats du duel").to_uppercase(), Vector2::new(results.x + 20.0, results.y + 16.0), 28.0);
    let col_x = |i: usize| results.x + 250.0 + i as f32 * 98.0;
    let header_y = results.y + 62.0;
    for (i, (label, color)) in [
        (tr("BAD", "RATÉ"), ui::BAD),
        (tr("GOOD", "BIEN"), Color::ORANGE),
        (tr("PERFECT", "PARFAIT"), ui::GOOD),
    ]
    .iter()
    .enumerate()
    {
        let width = assets.info_font.measure_text(label, 15.0, 1.0).x + 16.0;
        ui::draw_chip(d, &assets.info_font, Vector2::new(col_x(i) + 40.0 - width / 2.0, header_y), label, 15.0, *color);
    }
    for (row, (label, values)) in [
        (tr("You attack: damage dealt", "Tu attaques : dégâts infligés"), [("x0.5", ui::BAD), ("x1", ui::CREAM), ("x1.5", ui::GOOD)]),
        (tr("You defend: damage taken", "Tu défends : dégâts subis"), [("x1.5", ui::BAD), ("x1", ui::CREAM), ("x0.5", ui::GOOD)]),
    ]
    .iter()
    .enumerate()
    {
        let y = header_y + 40.0 + row as f32 * 52.0;
        ui::draw_wrapped(d, &assets.info_font, label, Vector2::new(results.x + 20.0, y + 2.0), 220.0, 16.0, Color::WHITE);
        for (i, (value, color)) in values.iter().enumerate() {
            let size = assets.alert_font.measure_text(value, 34.0, 2.0);
            d.draw_text_ex(&assets.alert_font, value, Vector2::new(col_x(i) + 40.0 - size.x / 2.0, y), 34.0, 2.0, *color);
        }
    }
    ui::draw_wrapped(
        d,
        &assets.info_font,
        tr(
            "Final damage = attack x defense. Wipe out every enemy to win!",
            "Dégâts finaux = attaque x défense. Élimine tous les ennemis pour gagner !",
        ),
        Vector2::new(results.x + 20.0, results.y + results.height - 58.0),
        results.width - 40.0,
        16.0,
        ui::GOLD,
    );

    // touches --------------------------------------------------------------------
    ui::draw_panel(d, controls);
    draw_card_title(d, assets, &tr("Controls", "Commandes").to_uppercase(), Vector2::new(controls.x + 20.0, controls.y + 16.0), 28.0);
    let key = |bind: Bind| settings::key_name(settings::key(bind));
    let rows = [
        (tr("Click", "Clic").to_string(), tr("Select, move, attack", "Choisir, bouger, attaquer")),
        (key(Bind::Cancel), tr("Cancel the action", "Annuler l'action")),
        (key(Bind::EndTurn), tr("End your turn", "Finir ton tour")),
        (key(Bind::Wait), tr("Unit waits", "L'unité attend")),
        (key(Bind::MapSpeed), tr("Map speed x1 / x2", "Vitesse carte x1 / x2")),
        (tr("Esc", "Échap").to_string(), tr("Pause", "Pause")),
    ];
    for (i, (cap, label)) in rows.iter().enumerate() {
        let col = (i / 3) as f32;
        let row = (i % 3) as f32;
        let x = controls.x + 20.0 + col * (controls.width / 2.0 - 6.0);
        let y = controls.y + 66.0 + row * 50.0;
        let width = ui::draw_keycap(d, &assets.info_font, Vector2::new(x, y), cap, 16.0);
        ui::draw_wrapped(d, &assets.info_font, label, Vector2::new(x + width + 10.0, y + 3.0), controls.width / 2.0 - width - 40.0, 15.0, Color::WHITE);
    }
    ui::draw_wrapped(
        d,
        &assets.info_font,
        tr("Change any key in Settings.", "Change les touches dans Réglages."),
        Vector2::new(controls.x + 20.0, controls.y + controls.height - 40.0),
        controls.width - 40.0,
        15.0,
        ui::MUTED,
    );
}

/// petite scène animée de chaque étape de la page Bases
fn draw_basics_scene(d: &mut RaylibDrawHandle, assets: &Assets, step: usize, scene: Rectangle, time: f32) {
    let cell = 30.0;
    match step {
        0 => {
            let origin = Vector2::new(scene.x + scene.width / 2.0 - cell * 2.5, scene.y + scene.height / 2.0 - cell * 1.5);
            draw_mini_grid(d, origin, 5, 3, cell);
            let pulse = 0.5 + 0.5 * (time * 4.0).sin();
            let me = mini_cell(origin, 2, 1, cell);
            d.draw_rectangle_rec(me, Color::new(255, 215, 0, (60.0 + 80.0 * pulse) as u8));
            draw_token(d, me, TOKEN_BLUE, true);
            draw_token(d, mini_cell(origin, 0, 2, cell), TOKEN_BLUE, false);
            d.draw_texture_ex(
                assets.cursor_texture(CursorType::Hover),
                Vector2::new(me.x + 18.0, me.y + 16.0),
                0.0,
                0.45,
                Color::WHITE,
            );
        }
        1 => {
            let origin = Vector2::new(scene.x + scene.width / 2.0 - cell * 3.5, scene.y + scene.height / 2.0 - cell * 1.5);
            draw_mini_grid(d, origin, 7, 3, cell);
            for col in 0..7 {
                for row in 0..3 {
                    if (col - 2_i32).abs() + (row - 1_i32).abs() <= 2 {
                        d.draw_rectangle_rec(mini_cell(origin, col, row, cell), Color::new(0, 100, 255, 110));
                    }
                }
            }
            // l'unité fait l'aller-retour vers la case visée
            let t = ((time * 0.8).sin() + 1.0) / 2.0;
            let from = mini_cell(origin, 2, 1, cell);
            let to = mini_cell(origin, 4, 1, cell);
            let moving = Rectangle { x: from.x + (to.x - from.x) * t, ..from };
            d.draw_rectangle_lines_ex(to, 2.0, Color::WHITE);
            draw_token(d, moving, TOKEN_BLUE, true);
        }
        2 => {
            let origin = Vector2::new(scene.x + scene.width / 2.0 - cell * 3.0, scene.y + scene.height / 2.0 - cell * 1.5);
            draw_mini_grid(d, origin, 6, 3, cell);
            let pulse = 0.5 + 0.5 * (time * 4.0).sin();
            let enemy = mini_cell(origin, 4, 1, cell);
            d.draw_rectangle_rec(enemy, Color::new(255, 0, 0, (110.0 + 70.0 * pulse) as u8));
            for (col, row) in [(3, 1), (4, 0), (4, 2)] {
                d.draw_rectangle_rec(mini_cell(origin, col, row, cell), Color::new(255, 255, 0, 120));
            }
            draw_token(d, mini_cell(origin, 1, 1, cell), TOKEN_BLUE, true);
            draw_token(d, enemy, TOKEN_RED, false);
            let ally = mini_cell(origin, 1, 2, cell);
            d.draw_rectangle_rec(ally, Color::new(0, 220, 0, 120));
            draw_token(d, ally, TOKEN_BLUE, false);
        }
        _ => {
            // barre de timing qui tourne
            let bar = Rectangle::new(scene.x + 20.0, scene.y + scene.height / 2.0 - 12.0, scene.width - 40.0, 24.0);
            d.draw_rectangle_rec(bar, Color::DARKGRAY);
            d.draw_rectangle_rec(Rectangle::new(bar.x + bar.width * 0.55, bar.y, bar.width * 0.2, bar.height), Color::YELLOW);
            d.draw_rectangle_rec(Rectangle::new(bar.x + bar.width * 0.635, bar.y, bar.width * 0.03, bar.height), Color::GREEN);
            let t = ((time * 2.2).sin() + 1.0) / 2.0;
            d.draw_rectangle((bar.x + bar.width * t) as i32 - 2, bar.y as i32 - 5, 4, bar.height as i32 + 10, Color::WHITE);
            let label = tr("PERFECT!", "PARFAIT !");
            let size = assets.alert_font.measure_text(label, 26.0, 2.0);
            let in_green = (0.635..=0.665).contains(&t);
            if in_green || (time * 2.0) as i32 % 3 == 0 {
                d.draw_text_ex(&assets.alert_font, label, Vector2::new(scene.x + scene.width / 2.0 - size.x / 2.0, bar.y - 36.0), 26.0, 2.0, ui::GOOD);
            }
        }
    }
}

fn draw_guide_minigames(d: &mut RaylibDrawHandle, assets: &Assets, mouse: Vector2, online: bool) {
    for (i, kind) in MiniGameKind::ALL.iter().enumerate() {
        let card = guide::minigame_rect(i);
        let hovered = !online && card.check_collision_point_rec(mouse);
        ui::draw_panel(d, card);
        if hovered {
            d.draw_rectangle_rounded_lines_ex(card, 0.04, 8, 2.0, ui::GOLD);
        }
        let illustration = Rectangle::new(card.x + 9.0, card.y + 10.0, card.width - 18.0, 96.0);
        draw_guide_illustration(d, *kind, illustration);
        draw_card_title(d, assets, &kind.name().to_uppercase(), Vector2::new(card.x + 16.0, card.y + 116.0), 28.0);
        let classes: Vec<&str> = kind.classes().iter().map(|c| c.name()).collect();
        d.draw_text_ex(&assets.info_font, &classes.join(" · "), Vector2::new(card.x + 16.0, card.y + 148.0), 14.0, 1.0, ui::MUTED);
        for (j, line) in kind.rule().iter().enumerate() {
            d.draw_text_ex(&assets.info_font, line, Vector2::new(card.x + 16.0, card.y + 170.0 + j as f32 * 19.0), 16.0, 1.0, Color::WHITE);
        }
        let label = if online { tr("Solo only", "Solo uniquement") } else { tr("Try it", "Essayer") };
        ui::draw_button(d, assets, guide::try_rect(card), label, Skin::Wood, mouse, !online);
    }
}

fn draw_guide_positioning(d: &mut RaylibDrawHandle, assets: &Assets, mouse: Vector2, online: bool) {
    for (i, card_info) in guide::position_cards().iter().enumerate() {
        let card = guide::position_rect(i);
        let hovered = !online && card.check_collision_point_rec(mouse);
        ui::draw_panel(d, card);
        if hovered {
            d.draw_rectangle_rounded_lines_ex(card, 0.04, 8, 2.0, ui::GOLD);
        }
        draw_card_title(d, assets, &card_info.title.to_uppercase(), Vector2::new(card.x + 18.0, card.y + 14.0), 28.0);
        let rows = card_info.rows.len() as f32;
        draw_guide_diagram(d, assets, card_info.rows, card.x + 22.0, card.y + 56.0 + (3.0 - rows) * DIAGRAM_CELL / 2.0);
        let text_x = card.x + 22.0 + 3.0 * DIAGRAM_CELL + 18.0;
        let text_width = card.x + card.width - 16.0 - text_x;
        let used = ui::draw_wrapped(d, &assets.info_font, card_info.lines[0], Vector2::new(text_x, card.y + 58.0), text_width, 16.0, Color::WHITE);
        ui::draw_wrapped(d, &assets.info_font, card_info.lines[1], Vector2::new(text_x, card.y + 64.0 + used), text_width, 16.0, ui::GOLD);
        let label = if online { tr("Solo only", "Solo uniquement") } else { tr("Try it", "Essayer") };
        ui::draw_button(d, assets, guide::try_rect(card), label, Skin::Wood, mouse, !online);
    }
    let last = guide::position_rect(5);
    ui::draw_wrapped(
        d,
        &assets.info_font,
        tr(
            "Effects add up: combine them! Under 30% HP, your green zone grows (comeback).",
            "Les effets s'additionnent : combine-les ! Sous 30% de PV, ta zone verte grandit.",
        ),
        Vector2::new(guide::position_rect(0).x + 4.0, last.y + last.height + 12.0),
        last.x + last.width - guide::position_rect(0).x,
        16.0,
        ui::CREAM,
    );
}

/// portrait d'une classe dans `rect` : la frame idle recadrée sur le personnage, ratio
/// gardé, posé en bas du rectangle. Tourné vers la gauche si `flip`
fn draw_class_portrait(d: &mut RaylibDrawHandle, assets: &Assets, class: UnitClass, rect: Rectangle, flip: bool) {
    let animation = assets_idle(assets, class);
    let frame = animation.animation_frame();
    let bounds = assets.portrait(class);
    let mut source = Rectangle::new(frame.x + bounds.x, frame.y + bounds.y, bounds.width, bounds.height);
    let scale = (rect.width / source.width).min(rect.height / source.height);
    let (width, height) = (source.width * scale, source.height * scale);
    if flip {
        source.width = -source.width;
    }
    d.draw_texture_pro(
        &animation.texture,
        source,
        Rectangle::new(rect.x + (rect.width - width) / 2.0, rect.y + rect.height - height, width, height),
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}

/// animation idle d'une classe (pour en dessiner une frame figée)
fn assets_idle(assets: &Assets, class: UnitClass) -> &Animation {
    match class {
        UnitClass::Soldier => &assets.soldier.idle,
        UnitClass::Cavalry => &assets.cavalry.idle,
        UnitClass::Assassin => &assets.assassin.idle,
        UnitClass::Longbowman => &assets.longbowman.idle,
        UnitClass::Mage => &assets.mage.idle,
        UnitClass::Priest => &assets.priest.idle,
        UnitClass::Wraith => &assets.wraith.idle,
        UnitClass::Banshee => &assets.banshee.idle,
        UnitClass::BloodKnight => &assets.blood_knight.idle,
        UnitClass::Ghoul => &assets.ghoul.idle,
        UnitClass::Skeleton => &assets.skeleton.idle,
        UnitClass::Necromancer => &assets.necromancer.idle,
    }
}



/// mini grille du guide : jetons bleus pour ton camp, rouges pour l'ennemi,
/// cases jaunes avec badge pour les choix de case
fn draw_guide_diagram(d: &mut impl RaylibDraw, assets: &Assets, rows: &[&str], x: f32, y: f32) {
    let cell = DIAGRAM_CELL;
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
                '2' => Some(("+25", ui::GOOD)),
                '1' => Some(("+12", ui::GOOD)),
                '0' => Some(("0", Color::WHITE)),
                'n' => Some(("-4", ui::BAD)),
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

// Multiplayer screen --------------------------------------------------------------

const LOBBY_GOLD: Color = Color::new(240, 196, 70, 255);
const LOBBY_MUTED: Color = Color::new(150, 160, 170, 255);
const LOBBY_HUMAN: Color = Color::new(120, 170, 255, 255);
const LOBBY_UNDEAD: Color = Color::new(230, 100, 100, 255);
const LOBBY_OPEN: Color = Color::new(120, 230, 120, 255);
// décalage de chaque colonne depuis le bord gauche des lignes
fn lobby_columns() -> [(&'static str, f32); 4] {
    [
        (tr("GAME", "PARTIE"), 0.0),
        (tr("HOST", "HÔTE"), 380.0),
        (tr("HOST FACTION", "FACTION DE L'HÔTE"), 620.0),
        (tr("PLAYERS", "JOUEURS"), 880.0),
    ]
}

pub fn draw_multiplayer_screen(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    delta_time: f32,
    hud: &HudRects,
    mouse_position: Vector2,
    browser: Option<&Browser>,
    selected: Option<IpAddr>,
    error: Option<&str>,
    connecting: bool,
) {
    draw_fullscreen_texture(d, &assets.menu_background_texture);

    // titre entre deux persos idle (pas ceux du guide)
    let title = tr("Multiplayer", "Multijoueur");
    let title_size = 48.0;
    let title_text_size = assets.hud_font.measure_text(title, title_size, 1.0);
    let header_middle = multiplayer::PANEL.y / 2.0;
    let title_pos = Vector2::new(
        SCREEN_WIDTH as f32 / 2.0 - title_text_size.x / 2.0,
        header_middle - title_text_size.y / 2.0,
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
    let sprite_top = header_middle - sprite_size / 2.0 + 10.0;
    let sprite_offset = title_text_size.x / 2.0 + 90.0;
    draw_idle_card(
        d,
        &mut assets.soldier.idle,
        SCREEN_WIDTH as f32 / 2.0 - sprite_offset,
        sprite_top,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.skeleton.idle,
        SCREEN_WIDTH as f32 / 2.0 + sprite_offset,
        sprite_top,
        sprite_size,
        delta_time,
    );

    // panneau de la liste
    let panel = multiplayer::PANEL;
    d.draw_rectangle_rec(panel, Color::new(18, 26, 36, 230));
    d.draw_rectangle_lines_ex(panel, 2.0, Color::new(255, 255, 255, 90));
    let left = panel.x + 24.0;
    let right = panel.x + panel.width - 24.0;
    // comme le guide : titres et boutons en hud_font, tout le texte en info_font
    let heading_font = &assets.hud_font;
    let font = &assets.info_font;

    d.draw_text_ex(
        heading_font,
        tr("Open games", "Parties ouvertes"),
        Vector2::new(left, 122.0),
        30.0,
        1.0,
        LOBBY_GOLD,
    );
    if let Some(browser) = browser {
        let count = browser.games().len();
        let plural = if count == 1 { "" } else { "s" };
        let found = trf!("{} game{} found", count, plural ; "{} partie{} trouvée{}", count, plural, plural);
        let size = font.measure_text(&found, 20.0, 1.0);
        let refresh = hud.btn_mp_refresh;
        d.draw_text_ex(
            font,
            &found,
            Vector2::new(
                refresh.x - 16.0 - size.x,
                refresh.y + refresh.height / 2.0 - size.y / 2.0,
            ),
            20.0,
            1.0,
            LOBBY_MUTED,
        );
        ui::draw_button(d, assets, refresh, tr("Refresh", "Actualiser"), Skin::Wood, mouse_position, true);
    }
    d.draw_line_ex(
        Vector2::new(left, 172.0),
        Vector2::new(right, 172.0),
        2.0,
        LOBBY_GOLD,
    );
    let columns_x = multiplayer::row_rect(0).x + 12.0;
    let columns = lobby_columns();
    for (label, offset) in columns {
        d.draw_text_ex(
            font,
            label,
            Vector2::new(columns_x + offset, 182.0),
            16.0,
            1.0,
            LOBBY_MUTED,
        );
    }

    // lignes, ou un message quand il n'y a rien à lister
    let message = match browser {
        None => Some(tr(
            "Port 6667 is already in use on this PC",
            "Le port 6667 est déjà utilisé sur ce PC",
        )),
        Some(browser) if browser.games().is_empty() => Some(tr(
            "Searching for games on your network...",
            "Recherche de parties sur ton réseau...",
        )),
        Some(_) => None,
    };
    if let Some(message) = message {
        let size = font.measure_text(message, 24.0, 1.0);
        d.draw_text_ex(
            font,
            message,
            Vector2::new(
                SCREEN_WIDTH as f32 / 2.0 - size.x / 2.0,
                panel.y + panel.height / 2.0,
            ),
            24.0,
            1.0,
            LOBBY_MUTED,
        );
    }
    let games = browser.map_or(&[][..], |b| b.games());
    for (i, lan_game) in games.iter().take(multiplayer::MAX_ROWS).enumerate() {
        let row = multiplayer::row_rect(i);
        let is_selected = selected == Some(lan_game.address);
        let background = if is_selected {
            Color::new(70, 60, 30, 220)
        } else if i % 2 == 0 {
            Color::new(32, 42, 54, 200)
        } else {
            Color::new(26, 34, 44, 200)
        };
        d.draw_rectangle_rec(row, background);
        if is_selected {
            d.draw_rectangle_lines_ex(row, 2.0, LOBBY_GOLD);
        }
        // partie pleine ou d'une autre version : toute la ligne est grisée
        let dim = |c: Color| {
            if !lan_game.info.is_joinable() {
                Color::new(c.r, c.g, c.b, 110)
            } else {
                c
            }
        };
        let text_y = row.y + row.height / 2.0 - 10.0;
        let cell = |offset: f32| Vector2::new(columns_x + offset, text_y);
        let info = &lan_game.info;
        d.draw_text_ex(
            font,
            &fit_text(font, &info.name, 20.0, 360.0),
            cell(columns[0].1),
            20.0,
            1.0,
            dim(Color::WHITE),
        );
        d.draw_text_ex(
            font,
            &fit_text(font, &info.host_name, 20.0, 220.0),
            cell(columns[1].1),
            20.0,
            1.0,
            dim(Color::WHITE),
        );
        let (icon, faction_name, faction_color) = match info.host_faction {
            Faction::Human => (&assets.soldier.idle, tr("Human", "Humains"), LOBBY_HUMAN),
            Faction::Undead => (&assets.skeleton.idle, tr("Undead", "Morts-vivants"), LOBBY_UNDEAD),
        };
        let faction_x = columns_x + columns[2].1;
        draw_animation_frame(
            d,
            icon,
            Rectangle::new(faction_x - 8.0, row.y - 4.0, 54.0, 54.0),
            dim(Color::WHITE),
        );
        d.draw_text_ex(
            font,
            faction_name,
            Vector2::new(faction_x + 46.0, text_y),
            20.0,
            1.0,
            dim(faction_color),
        );
        // autre version : on l'affiche à la place des joueurs, c'est ce qui bloque
        let (players, players_color) = if !info.is_compatible() {
            (format!("v{}", info.version), LOBBY_UNDEAD)
        } else if info.full {
            (tr("FULL", "PLEINE").to_string(), LOBBY_UNDEAD)
        } else {
            (String::from("1/2"), LOBBY_OPEN)
        };
        d.draw_text_ex(
            font,
            &players,
            cell(columns[3].1),
            20.0,
            1.0,
            dim(players_color),
        );
    }

    if let Some(lan_game) = games.iter().find(|g| Some(g.address) == selected) {
        let text = trf!("Selected: {}", lan_game.info.name ; "Sélection : {}", lan_game.info.name);
        let text = fit_text(font, &text, 20.0, panel.width - 48.0);
        let size = font.measure_text(&text, 20.0, 1.0);
        d.draw_text_ex(
            font,
            &text,
            Vector2::new(right - size.x, panel.y + panel.height - 24.0 - size.y),
            20.0,
            1.0,
            Color::WHITE,
        );
    }

    // connexion en cours : liste assombrie, message par-dessus
    if connecting {
        d.draw_rectangle_rec(panel, Color::new(0, 0, 0, 170));
        let name = games
            .iter()
            .find(|g| Some(g.address) == selected)
            .map_or(tr("the game", "la partie").to_string(), |g| g.info.name.clone());
        let text = fit_text(
            font,
            &trf!("Connecting to {}...", name ; "Connexion à {}...", name),
            30.0,
            panel.width - 48.0,
        );
        let size = font.measure_text(&text, 30.0, 1.0);
        d.draw_text_ex(
            font,
            &text,
            Vector2::new(
                panel.x + panel.width / 2.0 - size.x / 2.0,
                panel.y + panel.height / 2.0 - size.y / 2.0,
            ),
            30.0,
            1.0,
            Color::WHITE,
        );
    }
    if let Some(error) = error {
        let size = font.measure_text(error, 18.0, 1.0);
        d.draw_text_ex(
            font,
            error,
            Vector2::new(
                SCREEN_WIDTH as f32 / 2.0 - size.x / 2.0,
                panel.y + panel.height + 14.0,
            ),
            18.0,
            1.0,
            LOBBY_UNDEAD,
        );
    }

    let back_label = if connecting { tr("Cancel", "Annuler") } else { tr("Back", "Retour") };
    ui::draw_button(d, assets, hud.btn_mp_back, back_label, Skin::Wood, mouse_position, true);
    if !connecting {
        ui::draw_button(d, assets, hud.btn_mp_host, tr("Host game", "Créer"), Skin::Wood, mouse_position, true);
        // grisé tant qu'aucune partie n'est sélectionnée
        ui::draw_button(d, assets, hud.btn_mp_join, tr("Join", "Rejoindre"), Skin::Wood, mouse_position, selected.is_some());
    }

    d.draw_texture_ex(
        assets.cursor_texture(CursorType::Normal),
        Vector2::new(mouse_position.x, mouse_position.y),
        0.0,
        0.7,
        Color::WHITE,
    );
}

// Host game screen ---------------------------------------------------------------

pub fn draw_host_screen(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    delta_time: f32,
    hud: &HudRects,
    mouse_position: Vector2,
    form: &HostForm,
    waiting: bool,
) {
    draw_fullscreen_texture(d, &assets.menu_background_texture);

    // titre entre deux persos idle (ni ceux du guide, ni ceux de la liste)
    let title = tr("Host a game", "Créer une partie");
    let title_size = 48.0;
    let title_text_size = assets.hud_font.measure_text(title, title_size, 1.0);
    let header_middle = host::PANEL.y / 2.0;
    let title_pos = Vector2::new(
        SCREEN_WIDTH as f32 / 2.0 - title_text_size.x / 2.0,
        header_middle - title_text_size.y / 2.0,
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
    let sprite_top = header_middle - sprite_size / 2.0 + 10.0;
    let sprite_offset = title_text_size.x / 2.0 + 90.0;
    draw_idle_card(
        d,
        &mut assets.mage.idle,
        SCREEN_WIDTH as f32 / 2.0 - sprite_offset,
        sprite_top,
        sprite_size,
        delta_time,
    );
    draw_idle_card(
        d,
        &mut assets.banshee.idle,
        SCREEN_WIDTH as f32 / 2.0 + sprite_offset,
        sprite_top,
        sprite_size,
        delta_time,
    );

    let panel = host::PANEL;
    d.draw_rectangle_rec(panel, Color::new(18, 26, 36, 230));
    d.draw_rectangle_lines_ex(panel, 2.0, Color::new(255, 255, 255, 90));

    // champs texte : le champ actif a une bordure dorée
    for (field, label, text) in [
        (HostField::GameName, tr("GAME NAME", "NOM DE LA PARTIE"), &form.game_name),
        (HostField::PlayerName, tr("YOUR NAME", "TON NOM"), &form.player_name),
    ] {
        let rect = host::field_rect(field);
        let focused = form.focus == field;
        d.draw_text_ex(
            &assets.info_font,
            label,
            Vector2::new(rect.x, rect.y - 26.0),
            18.0,
            1.0,
            LOBBY_MUTED,
        );
        d.draw_rectangle_rec(rect, Color::new(10, 14, 20, 230));
        d.draw_rectangle_lines_ex(rect, 2.0, if focused { LOBBY_GOLD } else { LOBBY_MUTED });
        d.draw_text_ex(
            &assets.info_font,
            text,
            Vector2::new(rect.x + 14.0, rect.y + rect.height / 2.0 - 12.0),
            24.0,
            1.0,
            Color::WHITE,
        );
    }

    // choix de faction : carte avec le perso idle, la faction choisie en doré
    d.draw_text_ex(
        &assets.info_font,
        tr("YOUR FACTION", "TA FACTION"),
        Vector2::new(
            host::faction_rect(Faction::Human).x,
            host::faction_rect(Faction::Human).y - 26.0,
        ),
        18.0,
        1.0,
        LOBBY_MUTED,
    );
    for (faction, name, color) in [
        (Faction::Human, tr("Human", "Humains"), LOBBY_HUMAN),
        (Faction::Undead, tr("Undead", "Morts-vivants"), LOBBY_UNDEAD),
    ] {
        let rect = host::faction_rect(faction);
        let chosen = form.faction == faction;
        d.draw_rectangle_rec(
            rect,
            if chosen {
                Color::new(70, 60, 30, 220)
            } else {
                Color::new(26, 34, 44, 220)
            },
        );
        d.draw_rectangle_lines_ex(
            rect,
            if chosen { 3.0 } else { 1.0 },
            if chosen { LOBBY_GOLD } else { LOBBY_MUTED },
        );
        let animation = match faction {
            Faction::Human => &mut assets.soldier.idle,
            Faction::Undead => &mut assets.skeleton.idle,
        };
        draw_idle_card(
            d,
            animation,
            rect.x + rect.width / 2.0,
            rect.y - 10.0,
            150.0,
            delta_time,
        );
        let size = assets.info_font.measure_text(name, 26.0, 1.0);
        d.draw_text_ex(
            &assets.info_font,
            name,
            Vector2::new(
                rect.x + rect.width / 2.0 - size.x / 2.0,
                rect.y + rect.height - size.y - 14.0,
            ),
            26.0,
            1.0,
            color,
        );
    }

    // en attente d'un adversaire : formulaire assombri, message par-dessus
    if waiting {
        d.draw_rectangle_rec(panel, Color::new(0, 0, 0, 170));
        for (text, size, color, dy) in [
            (tr("Waiting for an opponent...", "En attente d'un adversaire..."), 30.0, Color::WHITE, -20.0),
            (
                tr("Your game is visible on the local network", "Ta partie est visible sur le réseau local"),
                18.0,
                LOBBY_MUTED,
                22.0,
            ),
        ] {
            let text_size = assets.info_font.measure_text(text, size, 1.0);
            d.draw_text_ex(
                &assets.info_font,
                text,
                Vector2::new(
                    panel.x + panel.width / 2.0 - text_size.x / 2.0,
                    panel.y + panel.height / 2.0 + dy,
                ),
                size,
                1.0,
                color,
            );
        }
    }
    if let Some(error) = form.error {
        let size = assets.info_font.measure_text(error, 18.0, 1.0);
        d.draw_text_ex(
            &assets.info_font,
            error,
            Vector2::new(
                SCREEN_WIDTH as f32 / 2.0 - size.x / 2.0,
                panel.y + panel.height + 14.0,
            ),
            18.0,
            1.0,
            LOBBY_UNDEAD,
        );
    }

    let back_label = if waiting { tr("Cancel", "Annuler") } else { tr("Back", "Retour") };
    ui::draw_button(d, assets, hud.btn_mp_back, back_label, Skin::Wood, mouse_position, true);
    if !waiting {
        // grisé tant qu'un nom manque
        ui::draw_button(d, assets, hud.btn_mp_join, tr("Host game", "Créer"), Skin::Wood, mouse_position, form.is_complete());
    }

    d.draw_texture_ex(
        assets.cursor_texture(CursorType::Normal),
        Vector2::new(mouse_position.x, mouse_position.y),
        0.0,
        0.7,
        Color::WHITE,
    );
}

/// coupe le texte avec "..." s'il dépasse `max_width`
fn fit_text(font: &Font, text: &str, size: f32, max_width: f32) -> String {
    if font.measure_text(text, size, 1.0).x <= max_width {
        return text.to_string();
    }
    let mut cut: String = text.to_string();
    while !cut.is_empty() && font.measure_text(&format!("{cut}..."), size, 1.0).x > max_width {
        cut.pop();
    }
    format!("{cut}...")
}



/// frame courante d'une animation dans `rect`, sans la faire avancer, ratio gardé
fn draw_animation_frame(
    d: &mut RaylibDrawHandle,
    animation: &Animation,
    rect: Rectangle,
    tint: Color,
) {
    let frame = animation.animation_frame();
    let scale = (rect.width / frame.width).min(rect.height / frame.height);
    let width = frame.width * scale;
    let height = frame.height * scale;
    d.draw_texture_pro(
        &animation.texture,
        frame,
        Rectangle::new(
            rect.x + (rect.width - width) / 2.0,
            rect.y + (rect.height - height) / 2.0,
            width,
            height,
        ),
        Vector2::new(0.0, 0.0),
        0.0,
        tint,
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

    let title = tr("Choose your faction", "Choisis ta faction");
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

    for (label, center_x) in [
        (tr("Human", "Humains"), human_center_x),
        (tr("Undead", "Morts-vivants"), undead_center_x),
    ] {
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
    waiting_rematch: bool,
) {
    d.clear_background(Color::BEIGE);
    tile_map.draw(d);
    let options = settings::get();

    // lignes de la grille (réglage)
    if options.grid_lines {
        let line = Color::new(0, 0, 0, 45);
        for y in (0..SCREEN_HEIGHT).step_by(TILE_SIZE as usize) {
            d.draw_rectangle(0, y, SCREEN_WIDTH, 1, line);
        }
        for x in (0..SCREEN_WIDTH).step_by(TILE_SIZE as usize) {
            d.draw_rectangle(x, 0, 1, SCREEN_HEIGHT, line);
        }
    }

    match current_turn {
        TurnPhase::PlayerTurn => {
            ui::draw_banner(d, assets, hud.banner_your_turn, tr("Your turn", "Ton tour"), false);
            ui::draw_button(d, assets, hud.btn_end_turn, tr("End turn", "Fin du tour"), Skin::Wood, mouse_position, true);
            if wait_button_visible {
                ui::draw_button(d, assets, hud.btn_wait, tr("Wait", "Attendre"), Skin::Wood, mouse_position, true);
            }
        }
        TurnPhase::EnemyTurn => {
            ui::draw_banner(d, assets, hud.banner_enemy_turn, tr("Enemy turn", "Tour ennemi"), true);
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

    if options.tile_bonuses {
        draw_position_scores(d, assets, position_scores);
    }
    if let Some(preview) = duel_preview.filter(|_| options.attack_preview) {
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
        GameMode::Victory => Some((tr("VICTORY", "VICTOIRE"), Color::GOLD)),
        GameMode::Defeat => Some((tr("DEFEAT", "DÉFAITE"), Color::RED)),
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
        if waiting_rematch {
            let text = tr("Waiting for opponent...", "En attente de l'adversaire...");
            let size = assets.alert_font.measure_text(text, 32.0, 1.0);
            d.draw_text_ex(
                &assets.alert_font,
                text,
                Vector2::new(
                    SCREEN_WIDTH as f32 / 2.0 - size.x / 2.0,
                    // au-dessus de VICTORY / DEFEAT, les boutons de fin sont en dessous
                    SCREEN_HEIGHT as f32 / 2.0 - text_size.y / 2.0 - size.y - 20.0,
                ),
                32.0,
                1.0,
                Color::WHITE,
            );
        }
        for (rect, label) in [
            (hud.btn_retry, tr("Retry", "Rejouer")),
            (hud.btn_back, tr("Main menu", "Menu")),
            (hud.btn_exit, tr("Exit", "Quitter")),
        ] {
            ui::draw_button(d, assets, rect, label, Skin::Wood, mouse_position, true);
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
        unit.class.name(),
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
        trf!("HP {}/{}", unit.hp_points, unit.hp_max_points ; "PV {}/{}", unit.hp_points, unit.hp_max_points),
        trf!("ATK {}  DEF {}", unit.attack_power, unit.defense ; "ATQ {}  DÉF {}", unit.attack_power, unit.defense),
        trf!(
            "RNG {}  MOV {}/{}", unit.attack_range, unit.move_points_remaining, unit.move_points ;
            "POR {}  DÉP {}/{}", unit.attack_range, unit.move_points_remaining, unit.move_points
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
    result_display: Option<&ResultDisplay>,
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
    // online : mon mini-jeu est fini (ou je n'en ai pas), on attend celui de l'adversaire
    if attacker.state == UnitState::MiniGame && minigame.is_none() {
        draw_waiting_for_opponent(d, assets);
    }
    if let Some(shown) = result_display {
        draw_timing_result(d, assets, shown, player_is_attacker);
    }
}

fn draw_waiting_for_opponent(d: &mut RaylibDrawHandle, assets: &Assets) {
    let text = tr("Waiting for opponent...", "En attente de l'adversaire...");
    let text_size = assets.alert_font.measure_text(text, 40.0, 1.0);
    d.draw_text_ex(
        &assets.alert_font,
        text,
        Vector2::new(
            SCREEN_WIDTH as f32 / 2.0 - text_size.x / 2.0,
            SCREEN_HEIGHT as f32 / 2.0 - text_size.y / 2.0 - 350.0,
        ),
        40.0,
        1.0,
        Color::WHITE,
    );
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
        unit.class.name(),
        Vector2::new(bar_x, bar_y + bar_height + 10.0),
        24.0,
        1.0,
        Color::BLACK,
    );
}

fn draw_timing_result(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    shown: &ResultDisplay,
    player_is_attacker: bool,
) {
    let label = |r: TimingResult| match r {
        TimingResult::Bad => (tr("BAD", "RATÉ"), Color::RED),
        TimingResult::Good => (tr("GOOD", "BIEN"), Color::ORANGE),
        TimingResult::Perfect => (tr("PERFECT", "PARFAIT"), Color::LIME),
    };
    // le résultat du joueur d'abord, celui d'en face ensuite
    let (mine, theirs) = match shown.parry {
        Some(parry) if !player_is_attacker => (parry, Some(shown.attack)),
        parry => (shown.attack, parry),
    };
    let (mine_text, color) = label(mine);
    let full_text = match theirs {
        Some(t) => format!("{} vs {}  x{:.2}", mine_text, label(t).0, shown.multiplier),
        None => format!("{} x{:.1}", mine_text, shown.multiplier),
    };

    let text_size = assets.alert_font.measure_text(&full_text, 40.0, 1.0);
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

fn score_color(score: i32) -> Color {
    match score {
        s if s > 0 => ui::GOOD,
        s if s < 0 => ui::BAD,
        _ => Color::new(200, 190, 170, 255),
    }
}

/// petit badge +12 / -4 dans le coin de chaque case d'attaque possible
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
        let font_size = 17.0;
        let size = assets.info_font.measure_text(&text, font_size, 1.0);
        let rect = Rectangle::new(
            (x * TILE_SIZE) as f32 + 2.0,
            (y * TILE_SIZE) as f32 + 2.0,
            size.x + 8.0,
            size.y + 2.0,
        );
        d.draw_rectangle_rounded(rect, 0.4, 6, Color::new(16, 14, 18, 220));
        d.draw_text_ex(
            &assets.info_font,
            &text,
            Vector2::new(rect.x + 4.0, rect.y + 1.0),
            font_size,
            1.0,
            score_color(*score),
        );
    }
}





/// carte d'aperçu du duel : qui attaque qui, PV de la cible après le coup, dégâts selon
/// le résultat du mini-jeu, bonus du placement et ses raisons en pastilles
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
    let advantage = c.advantage();

    let width = 380.0;
    let pad = 16.0;
    let inner = width - pad * 2.0;
    let chip_size = 15.0;
    let chip_height = chip_size * 1.45 + 6.0;
    // pastilles des facteurs, réparties sur des lignes
    let labels: Vec<(String, Color)> = c
        .factors
        .iter()
        .map(|f| {
            let color = if f.favors_attacker { ui::GOOD } else { ui::BAD };
            let sign = if f.favors_attacker { "+" } else { "-" };
            (format!("{sign} {}", f.label(true)), color)
        })
        .collect();
    let chip_width = |text: &str| assets.info_font.measure_text(text, chip_size, 1.0).x + chip_size * 1.1;
    // répartit les pastilles en lignes qui tiennent dans la largeur
    let mut chip_rows: Vec<Vec<usize>> = Vec::new();
    let mut row_width = 0.0;
    for (i, (text, _)) in labels.iter().enumerate() {
        let w = chip_width(text);
        if chip_rows.is_empty() || row_width + w > inner {
            chip_rows.push(Vec::new());
            row_width = 0.0;
        }
        chip_rows.last_mut().unwrap().push(i);
        row_width += w + 6.0;
    }
    let best_tile_height = if preview.from_best_tile { 20.0 } else { 0.0 };
    let height = pad + 30.0 + best_tile_height + 10.0 + 22.0 + 16.0 + 14.0 + 52.0 + 14.0 + 30.0
        + chip_rows.len() as f32 * chip_height
        + pad;

    // côté opposé à la cible pour pas cacher le combat, sous la bannière de tour à droite
    let panel = if defender.grid_x * TILE_SIZE < SCREEN_WIDTH / 2 {
        Rectangle::new(SCREEN_WIDTH as f32 - width - 12.0, 84.0, width, height)
    } else {
        Rectangle::new(12.0, 62.0, width, height)
    };
    d.draw_rectangle_rounded(panel, 0.06, 8, Color::new(16, 14, 18, 236));
    d.draw_rectangle_rounded_lines_ex(panel, 0.06, 8, 2.5, score_color(advantage));

    let x = panel.x + pad;
    let mut y = panel.y + pad;

    // en-tête : attaquant  >  cible
    let name_size = 26.0;
    d.draw_text_ex(&assets.alert_font, attacker.class.name(), Vector2::new(x, y), name_size, 2.0, ui::CREAM);
    let target_name = defender.class.name();
    let target_width = assets.alert_font.measure_text(target_name, name_size, 2.0).x;
    d.draw_text_ex(&assets.alert_font, target_name, Vector2::new(x + inner - target_width, y), name_size, 2.0, Color::new(255, 150, 140, 255));
    draw_crossed_swords(d, Vector2::new(x + inner / 2.0, y + 11.0));
    y += 30.0;
    if preview.from_best_tile {
        d.draw_text_ex(
            &assets.info_font,
            tr("from the best tile (white frame)", "depuis la meilleure case (cadre blanc)"),
            Vector2::new(x, y),
            14.0,
            1.0,
            ui::MUTED,
        );
        y += best_tile_height;
    }
    y += 10.0;

    // PV de la cible : ce qu'un GOOD enlève en rouge vif, le bonus d'un PERFECT plus sombre
    let after_good = (hp - good).max(0);
    let hp_text = trf!("HP {} > {}", hp, after_good ; "PV {} > {}", hp, after_good);
    d.draw_text_ex(&assets.info_font, &hp_text, Vector2::new(x, y), 17.0, 1.0, Color::WHITE);
    let ko = if hp - bad <= 0 {
        Some((tr("SURE KO", "KO ASSURÉ"), Color::GOLD))
    } else if hp - good <= 0 {
        Some((tr("KO", "KO"), Color::GOLD))
    } else if hp - perfect <= 0 {
        Some((tr("KO if PERFECT", "KO si PARFAIT"), Color::ORANGE))
    } else {
        None
    };
    if let Some((text, color)) = ko {
        let w = chip_width(text);
        ui::draw_chip(d, &assets.info_font, Vector2::new(x + inner - w, y - 3.0), text, chip_size, color);
    }
    y += 22.0;
    let max_hp = defender.hp_max_points.max(1) as f32;
    let px = |value: i32| x + inner * (value.max(0) as f32 / max_hp);
    let bar_h = 16.0;
    d.draw_rectangle_rec(Rectangle::new(x, y, inner, bar_h), Color::new(50, 46, 44, 255));
    // vert : ce qui reste même sur un PARFAIT. Rouge vif : ce qu'enlève un BIEN.
    // Rouge sombre qui clignote : ce qu'un PARFAIT enlève en plus
    d.draw_rectangle_rec(Rectangle::new(x, y, px(hp - perfect) - x, bar_h), Color::new(90, 200, 90, 255));
    d.draw_rectangle_rec(Rectangle::new(px(hp - good), y, px(hp) - px(hp - good), bar_h), Color::new(235, 80, 60, 255));
    let pulse = 0.5 + 0.5 * (d.get_time() as f32 * 5.0).sin();
    d.draw_rectangle_rec(
        Rectangle::new(px(hp - perfect), y, px(hp - good) - px(hp - perfect), bar_h),
        Color::new((120.0 + 60.0 * pulse) as u8, 30, 30, 255),
    );
    d.draw_rectangle_lines_ex(Rectangle::new(x, y, inner, bar_h), 1.5, Color::new(255, 255, 255, 140));
    y += bar_h + 14.0;

    // dégâts selon le résultat du mini-jeu
    let cell_gap = 8.0;
    let cell_w = (inner - cell_gap * 2.0) / 3.0;
    for (i, (label, value, color)) in [
        (tr("BAD", "RATÉ"), bad, ui::BAD),
        (tr("GOOD", "BIEN"), good, Color::ORANGE),
        (tr("PERFECT", "PARFAIT"), perfect, ui::GOOD),
    ]
    .iter()
    .enumerate()
    {
        let cell = Rectangle::new(x + i as f32 * (cell_w + cell_gap), y, cell_w, 52.0);
        d.draw_rectangle_rounded(cell, 0.18, 6, Color::new(32, 28, 32, 255));
        d.draw_rectangle_rec(Rectangle::new(cell.x + 6.0, cell.y, cell.width - 12.0, 3.0), *color);
        let label_w = assets.info_font.measure_text(label, 13.0, 1.0).x;
        d.draw_text_ex(&assets.info_font, label, Vector2::new(cell.x + cell.width / 2.0 - label_w / 2.0, cell.y + 7.0), 13.0, 1.0, *color);
        let number = value.to_string();
        let number_w = assets.alert_font.measure_text(&number, 28.0, 2.0).x;
        d.draw_text_ex(&assets.alert_font, &number, Vector2::new(cell.x + cell.width / 2.0 - number_w / 2.0, cell.y + 22.0), 28.0, 2.0, ui::CREAM);
    }
    y += 52.0 + 14.0;

    // bonus du placement, puis ses raisons
    let advantage_text = if advantage > 0 {
        format!("+{advantage}%")
    } else {
        format!("{advantage}%")
    };
    d.draw_text_ex(&assets.alert_font, &advantage_text, Vector2::new(x, y), 30.0, 2.0, score_color(advantage));
    let advantage_w = assets.alert_font.measure_text(&advantage_text, 30.0, 2.0).x;
    let caption = if labels.is_empty() {
        tr("no placement bonus", "aucun bonus de placement")
    } else {
        tr("damage from placement", "dégâts grâce au placement")
    };
    d.draw_text_ex(&assets.info_font, caption, Vector2::new(x + advantage_w + 10.0, y + 7.0), 15.0, 1.0, ui::MUTED);
    y += 30.0;
    for row in &chip_rows {
        let mut chip_x = x;
        for &i in row {
            let (text, color) = &labels[i];
            chip_x += ui::draw_chip(d, &assets.info_font, Vector2::new(chip_x, y), text, chip_size, *color) + 6.0;
        }
        y += chip_height;
    }
}

/// deux épées croisées (entre l'attaquant et la cible)
fn draw_crossed_swords(d: &mut RaylibDrawHandle, center: Vector2) {
    let blade = Color::new(220, 220, 230, 255);
    let hilt = Color::new(200, 150, 60, 255);
    for dir in [Vector2::new(1.0, 1.0), Vector2::new(-1.0, 1.0)] {
        let tip = center - dir * 10.0;
        let base = center + dir * 7.0;
        d.draw_line_ex(tip, base, 3.0, blade);
        let side = Vector2::new(-dir.y, dir.x) * 4.0;
        d.draw_line_ex(base - side, base + side, 3.0, hilt);
        d.draw_line_ex(base, base + dir * 4.0, 3.0, hilt);
    }
}

// combat : rappel des facteurs ------------------------------------------

/// rappel des effets du placement pendant le duel : une pastille par effet, en bas à
/// gauche, verte s'il aide le joueur, rouge s'il le gêne
fn draw_combat_factors(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    conditions: &DuelConditions,
    player_is_attacker: bool,
) {
    let size = 18.0;
    let line_height = size * 1.45 + 10.0;
    let mut y = SCREEN_HEIGHT as f32 - 16.0 - conditions.factors.len() as f32 * line_height;
    for factor in &conditions.factors {
        let good_for_me = factor.favors_attacker == player_is_attacker;
        let (sign, color) = if good_for_me {
            ("+", ui::GOOD)
        } else {
            ("-", ui::BAD)
        };
        let text = format!("{sign} {}", factor.label(player_is_attacker));
        ui::draw_chip(d, &assets.info_font, Vector2::new(16.0, y), &text, size, color);
        y += line_height;
    }
}

// illustrations des mini-jeux dans le guide ------------------------------------

/// vue figée d'un mini-jeu dans un cadre, mêmes couleurs que le vrai jeu
fn draw_guide_illustration(d: &mut impl RaylibDraw, kind: MiniGameKind, rect: Rectangle) {
    d.draw_rectangle_rec(rect, Color::new(10, 10, 10, 220));
    d.draw_rectangle_lines_ex(rect, 2.0, Color::new(255, 255, 255, 90));
    let center = Vector2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);

    match kind {
        MiniGameKind::Timing => {
            let bar = Rectangle::new(rect.x + 20.0, center.y - 10.0, rect.width - 40.0, 22.0);
            d.draw_rectangle_rec(bar, Color::DARKGRAY);
            d.draw_rectangle_rec(Rectangle::new(bar.x + bar.width * 0.5, bar.y, bar.width * 0.22, bar.height), Color::YELLOW);
            d.draw_rectangle_rec(Rectangle::new(bar.x + bar.width * 0.595, bar.y, bar.width * 0.03, bar.height), Color::GREEN);
            d.draw_rectangle((bar.x + bar.width * 0.3) as i32 - 2, bar.y as i32 - 5, 4, bar.height as i32 + 10, Color::WHITE);
            draw_guide_arrow(
                d,
                Vector2::new(bar.x + bar.width * 0.3 + 8.0, bar.y - 14.0),
                Vector2::new(bar.x + bar.width * 0.48, bar.y - 14.0),
                Color::new(255, 255, 255, 160),
            );
        }
        MiniGameKind::Target => {
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
        MiniGameKind::Charge => {
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
        MiniGameKind::Sequence => {
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
        MiniGameKind::Runes => {
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
        MiniGameKind::Channel => {
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
        MiniGameKind::Heartbeat => {
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
            let spacing = (track.x + track.width - 20.0 - line_x) / 3.0;
            for drop_x in [line_x + spacing, line_x + spacing * 2.0, line_x + spacing * 3.0] {
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

// Settings screen -----------------------------------------------------------------

pub fn draw_settings_screen(
    d: &mut RaylibDrawHandle,
    assets: &Assets,
    hud: &HudRects,
    mouse_position: Vector2,
    capturing: Option<Bind>,
) {
    use crate::screens::settings as layout;
    draw_fullscreen_texture(d, &assets.menu_background_texture);
    d.draw_rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::new(0, 0, 0, 90));
    ui::draw_title(d, &assets.hud_font, tr("Settings", "Réglages"), SCREEN_WIDTH as f32 / 2.0, 20.0, 46.0);
    let current = settings::get();

    // général ----------------------------------------------------------------------
    ui::draw_panel(d, layout::GENERAL);
    draw_card_title(d, assets, &tr("General", "Général").to_uppercase(), Vector2::new(layout::GENERAL.x + 28.0, layout::GENERAL.y + 18.0), 30.0);
    for (i, toggle) in layout::toggles(&current).iter().enumerate() {
        let pos = layout::toggle_title_pos(i);
        d.draw_text_ex(&assets.info_font, toggle.title, pos, 20.0, 1.0, Color::WHITE);
        d.draw_text_ex(&assets.info_font, toggle.detail, pos + Vector2::new(0.0, 25.0), 15.0, 1.0, ui::MUTED);
        ui::draw_segmented(d, &assets.info_font, layout::toggle_rect(i), &toggle.options, toggle.active, mouse_position);
    }

    // touches ----------------------------------------------------------------------
    let panel = layout::CONTROLS;
    ui::draw_panel(d, panel);
    draw_card_title(d, assets, &tr("Controls", "Touches").to_uppercase(), Vector2::new(panel.x + 28.0, panel.y + 18.0), 30.0);
    ui::draw_button(d, assets, layout::reset_rect(), tr("Default keys", "Par défaut"), Skin::Wood, mouse_position, capturing.is_none());

    let label_x = panel.x + 28.0;
    let heading = |d: &mut RaylibDrawHandle, text: &str, y: f32| {
        d.draw_text_ex(&assets.alert_font, text, Vector2::new(label_x, y), 22.0, 2.0, ui::GOLD);
        d.draw_rectangle(label_x as i32, (y + 24.0) as i32, (panel.width - 56.0) as i32, 1, Color::new(240, 196, 70, 90));
    };
    heading(d, &tr("On the map", "Sur la carte").to_uppercase(), layout::map_heading_y());
    // touches fixes, grisées
    let fixed_y = layout::key_rows()[0].1 - 2.0 * layout::KEY_ROW;
    for (i, (label, key)) in [
        (tr("Select, move, attack", "Choisir, bouger, attaquer"), tr("Left click", "Clic gauche")),
        (tr("Pause / back", "Pause / retour"), tr("Esc", "Échap")),
    ]
    .iter()
    .enumerate()
    {
        let y = fixed_y + i as f32 * layout::KEY_ROW;
        d.draw_text_ex(&assets.info_font, label, Vector2::new(label_x, y + 3.0), 16.0, 1.0, ui::MUTED);
        let rect = layout::key_rect(y);
        let width = assets.info_font.measure_text(key, 14.0, 1.0).x;
        d.draw_text_ex(&assets.info_font, key, Vector2::new(rect.x + rect.width / 2.0 - width / 2.0, y + 4.0), 14.0, 1.0, ui::MUTED);
    }
    heading(d, &tr("In duels", "En duel").to_uppercase(), layout::duel_heading_y());
    d.draw_text_ex(
        &assets.info_font,
        tr("Mouse click and arrow keys always work too.", "Le clic et les flèches marchent toujours aussi."),
        Vector2::new(label_x, layout::duel_heading_y() + layout::KEY_ROW + 4.0),
        14.0,
        1.0,
        ui::MUTED,
    );
    let blink = (d.get_time() * 3.0) as i32 % 2 == 0;
    for (bind, y) in layout::key_rows() {
        d.draw_text_ex(&assets.info_font, bind.label(), Vector2::new(label_x, y + 3.0), 16.0, 1.0, Color::WHITE);
        let rect = layout::key_rect(y);
        let hovered = capturing.is_none() && rect.check_collision_point_rec(mouse_position);
        if capturing == Some(bind) {
            d.draw_rectangle_rounded(rect, 0.3, 6, Color::new(92, 70, 28, 240));
            d.draw_rectangle_rounded_lines_ex(rect, 0.3, 6, 2.0, if blink { ui::GOLD } else { ui::CREAM });
            let text = tr("Press a key...", "Appuie sur une touche...");
            let width = assets.info_font.measure_text(text, 13.0, 1.0).x;
            d.draw_text_ex(&assets.info_font, text, Vector2::new(rect.x + rect.width / 2.0 - width / 2.0, rect.y + 5.0), 13.0, 1.0, Color::WHITE);
        } else {
            if hovered {
                d.draw_rectangle_rounded(rect, 0.3, 6, Color::new(255, 230, 170, 30));
            }
            let name = settings::key_name(current.key(bind));
            let width = assets.info_font.measure_text(&name, 14.0, 1.0).x + 14.0;
            ui::draw_keycap(d, &assets.info_font, Vector2::new(rect.x + rect.width / 2.0 - width / 2.0, rect.y), &name, 14.0);
        }
    }

    ui::draw_button(d, assets, hud.btn_guide_back, tr("Back", "Retour"), Skin::Wood, mouse_position, true);
    draw_cursor(d, assets, mouse_position);
}

// Practice screen -----------------------------------------------------------------

pub fn draw_practice_screen(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    delta_time: f32,
    hud: &HudRects,
    mouse_position: Vector2,
    practice: &Practice,
) {
    match &practice.phase {
        PracticePhase::Setup => draw_practice_setup(d, assets, delta_time, hud, mouse_position, practice),
        PracticePhase::Playing { minigame, .. } => {
            draw_practice_duel(d, assets, delta_time, practice);
            minigame.draw(d, assets);
            let hint = tr("Esc: stop", "Échap : arrêter");
            d.draw_text_ex(&assets.info_font, hint, Vector2::new(SCREEN_WIDTH as f32 - 170.0, 16.0), 16.0, 1.0, Color::new(255, 255, 255, 180));
        }
        PracticePhase::Result { result } => {
            draw_practice_duel(d, assets, delta_time, practice);
            draw_practice_result(d, assets, mouse_position, practice, *result);
        }
    }
    draw_cursor(d, assets, mouse_position);
}

fn draw_practice_setup(
    d: &mut RaylibDrawHandle,
    assets: &mut Assets,
    delta_time: f32,
    hud: &HudRects,
    mouse: Vector2,
    practice: &Practice,
) {
    draw_fullscreen_texture(d, &assets.menu_background_texture);
    d.draw_rectangle(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::new(0, 0, 0, 100));
    ui::draw_title(d, &assets.hud_font, tr("Training ground", "Terrain d'entraînement"), SCREEN_WIDTH as f32 / 2.0, 16.0, 44.0);
    let assets = &*assets;
    let left = practice::LEFT;

    // 1. l'unité ------------------------------------------------------------------
    ui::draw_panel(d, left);
    let heading = |d: &mut RaylibDrawHandle, text: &str, y: f32| {
        draw_card_title(d, assets, text, Vector2::new(left.x + 22.0, y), 24.0);
    };
    heading(d, &trf!("1  Your unit: {}", practice.class.name() ; "1  Ton unité : {}", practice.class.name()).to_uppercase(), left.y + 16.0);
    for (i, class) in practice::all_classes().enumerate() {
        let rect = practice::class_rect(i);
        let selected = class == practice.class;
        let hovered = rect.check_collision_point_rec(mouse);
        d.draw_rectangle_rounded(rect, 0.12, 6, if selected { Color::new(92, 70, 28, 240) } else { Color::new(30, 28, 34, 240) });
        let border = if selected {
            ui::GOLD
        } else if hovered {
            ui::CREAM
        } else {
            ui::PANEL_LINE
        };
        d.draw_rectangle_rounded_lines_ex(rect, 0.12, 6, if selected { 3.0 } else { 1.5 }, border);
        let sprite = Rectangle::new(rect.x + 8.0, rect.y + 6.0, rect.width - 16.0, rect.height - 12.0);
        draw_class_portrait(d, assets, class, sprite, false);
    }

    // 2. le rôle -------------------------------------------------------------------
    heading(d, &tr("2  Role", "2  Rôle").to_uppercase(), left.y + 238.0);
    let role_index = (practice.role == Role::Defend) as usize;
    ui::draw_segmented(
        d,
        &assets.info_font,
        practice::role_rect(),
        &[tr("I attack", "J'attaque"), tr("I defend", "Je défends")],
        role_index,
        mouse,
    );
    let role_text = match practice.role {
        Role::Attack => tr("Your mini-game sets the damage you deal.", "Ton mini-jeu décide des dégâts infligés."),
        Role::Defend => tr("Your mini-game is a parry: less damage taken.", "Ton mini-jeu est une parade : moins de dégâts subis."),
    };
    ui::draw_wrapped(d, &assets.info_font, role_text, Vector2::new(practice::role_rect().x + 336.0, practice::role_rect().y - 2.0), 180.0, 14.0, ui::MUTED);

    // 3. le placement --------------------------------------------------------------
    heading(d, &tr("3  Placement", "3  Placement").to_uppercase(), left.y + 326.0);
    let hint = tr("click a tile: ally > enemy > empty", "clic : allié > ennemi > vide");
    let hint_width = assets.info_font.measure_text(hint, 14.0, 1.0).x;
    d.draw_text_ex(&assets.info_font, hint, Vector2::new(left.x + left.width - 22.0 - hint_width, left.y + 332.0), 14.0, 1.0, ui::MUTED);
    let (attacker_class, defender_class) = practice.duel_classes();
    let attacker_tile = practice.attacker_tile();
    let target_tile = practice.target_tile();
    for y in 0..practice::BOARD_ROWS {
        for x in 0..practice::BOARD_COLS {
            let rect = practice::cell_rect(x, y);
            let shade = if (x + y) % 2 == 0 { Color::new(74, 100, 62, 255) } else { Color::new(66, 90, 56, 255) };
            d.draw_rectangle_rec(rect, shade);
            d.draw_rectangle_lines_ex(rect, 1.0, Color::new(0, 0, 0, 80));
            let tile = (x as i32, y as i32);
            let free = tile != attacker_tile && tile != target_tile;
            if free && rect.check_collision_point_rec(mouse) {
                d.draw_rectangle_rec(rect, Color::new(255, 255, 255, 40));
            }
            match practice.board[y][x] {
                Cell::Ally => draw_token(d, rect, TOKEN_BLUE, false),
                Cell::Enemy => draw_token(d, rect, TOKEN_RED, false),
                Cell::Empty => {}
            }
        }
    }
    // l'attaquant (à gauche) et la cible, en sprites. Le joueur a un anneau doré
    let me_attacking = practice.role == Role::Attack;
    for (class, tile, flip, mine) in [
        (attacker_class, attacker_tile, false, me_attacking),
        (defender_class, target_tile, true, !me_attacking),
    ] {
        let cell = practice::cell_rect(tile.0 as usize, tile.1 as usize);
        let mine_faction = class.faction() == practice.class.faction();
        d.draw_rectangle_rec(cell, if mine_faction { Color::new(70, 130, 255, 90) } else { Color::new(220, 60, 60, 90) });
        if mine {
            d.draw_rectangle_lines_ex(cell, 3.0, ui::GOLD);
        }
        let sprite = Rectangle::new(cell.x - 6.0, cell.y - 18.0, cell.width + 12.0, cell.height + 14.0);
        draw_class_portrait(d, assets, class, sprite, flip);
    }
    // options à droite du plateau
    let options = [
        (practice.range_allowed().then_some(tr("Shoot from range", "Tirer de loin")), practice.at_range),
        (Some(tr("Me: low HP", "Moi : PV bas")), practice.me_low),
        (Some(tr("Them: low HP", "Lui : PV bas")), practice.them_low),
    ];
    for (i, (label, checked)) in options.iter().enumerate() {
        let Some(label) = label else {
            continue;
        };
        let rect = practice::option_rect(i);
        let hovered = rect.check_collision_point_rec(mouse);
        let boxed = Rectangle::new(rect.x, rect.y + 9.0, 22.0, 22.0);
        d.draw_rectangle_rounded(boxed, 0.25, 4, if *checked { Color::new(92, 70, 28, 255) } else { Color::new(30, 28, 34, 255) });
        d.draw_rectangle_rounded_lines_ex(boxed, 0.25, 4, 2.0, if *checked || hovered { ui::GOLD } else { ui::PANEL_LINE });
        if *checked {
            d.draw_line_ex(Vector2::new(boxed.x + 5.0, boxed.y + 11.0), Vector2::new(boxed.x + 10.0, boxed.y + 17.0), 3.0, ui::CREAM);
            d.draw_line_ex(Vector2::new(boxed.x + 10.0, boxed.y + 17.0), Vector2::new(boxed.x + 18.0, boxed.y + 5.0), 3.0, ui::CREAM);
        }
        d.draw_text_ex(&assets.info_font, label, Vector2::new(rect.x + 32.0, rect.y + 11.0), 16.0, 1.0, Color::WHITE);
    }
    ui::draw_button(d, assets, practice::option_rect(3), tr("Clear", "Vider"), Skin::Wood, mouse, true);

    // à droite : le mini-jeu, la barre obtenue et pourquoi ----------------------------
    let right = practice::RIGHT;
    ui::draw_panel(d, right);
    let kind = practice.kind();
    draw_card_title(d, assets, &kind.name().to_uppercase(), Vector2::new(right.x + 24.0, right.y + 18.0), 38.0);
    for (j, line) in kind.rule().iter().enumerate() {
        d.draw_text_ex(&assets.info_font, line, Vector2::new(right.x + 24.0, right.y + 66.0 + j as f32 * 21.0), 17.0, 1.0, Color::WHITE);
    }
    draw_guide_illustration(d, kind, Rectangle::new(right.x + right.width - 274.0, right.y + 16.0, 250.0, 96.0));

    let side = practice.my_side();
    let base = DuelSide::base();
    draw_card_title(d, assets, &tr("Your bar", "Ta barre").to_uppercase(), Vector2::new(right.x + 24.0, right.y + 132.0), 24.0);
    // grande barre d'exemple, zones centrées, vies en carrés rouges à gauche
    let bar = Rectangle::new(right.x + 70.0, right.y + 172.0, right.width - 94.0, 26.0);
    d.draw_rectangle_rec(bar, Color::DARKGRAY);
    d.draw_rectangle_rec(Rectangle::new(bar.x + bar.width * (0.5 - side.window / 2.0), bar.y, bar.width * side.window, bar.height), Color::YELLOW);
    if side.precision > 0.0 {
        let w = (bar.width * side.precision).max(2.0);
        d.draw_rectangle_rec(Rectangle::new(bar.x + bar.width / 2.0 - w / 2.0, bar.y, w, bar.height), Color::GREEN);
    }
    // repère de la barre de base, pour voir ce qui a changé
    for edge in [0.5 - base.window / 2.0, 0.5 + base.window / 2.0] {
        d.draw_rectangle((bar.x + bar.width * edge) as i32 - 1, bar.y as i32 - 6, 2, bar.height as i32 + 12, Color::new(255, 255, 255, 150));
    }
    for i in 0..side.lives {
        d.draw_rectangle((bar.x - 22.0 - i as f32 * 18.0) as i32, (bar.y + 6.0) as i32, 14, 14, Color::RED);
    }
    let pct = |v: f32| (v * 100.0).round() as i32;
    let delta_color = |delta: i32| match delta {
        x if x > 0 => ui::GOOD,
        x if x < 0 => ui::BAD,
        _ => ui::CREAM,
    };
    let window_delta = pct(side.window) - pct(base.window);
    let precision_delta = pct(side.precision) - pct(base.precision);
    // tempo : plus bas = plus calme = mieux
    let tempo_delta = -((side.tempo - 1.0) * 100.0).round() as i32;
    let stats = [
        (trf!("Yellow {}%", pct(side.window) ; "Jaune {}%", pct(side.window)), delta_color(window_delta)),
        (trf!("Green {}%", pct(side.precision) ; "Vert {}%", pct(side.precision)), delta_color(precision_delta)),
        (trf!("Speed x{:.1}", side.tempo ; "Vitesse x{:.1}", side.tempo), delta_color(tempo_delta)),
        (trf!("Lives {}", side.lives ; "Vies {}", side.lives), delta_color(side.lives as i32)),
    ];
    let mut chip_x = right.x + 24.0;
    for (text, color) in &stats {
        chip_x += ui::draw_chip(d, &assets.info_font, Vector2::new(chip_x, right.y + 214.0), text, 15.0, *color) + 8.0;
    }

    draw_card_title(d, assets, &tr("Effects", "Effets").to_uppercase(), Vector2::new(right.x + 24.0, right.y + 262.0), 24.0);
    let conditions = practice.conditions();
    let mut y = right.y + 298.0;
    if conditions.factors.is_empty() {
        ui::draw_wrapped(
            d,
            &assets.info_font,
            tr(
                "None yet. Place allies (blue) or enemies (red) around the duel, or try low HP.",
                "Aucun pour l'instant. Place des alliés (bleu) ou des ennemis (rouge) autour du duel, ou essaie les PV bas.",
            ),
            Vector2::new(right.x + 24.0, y),
            right.width - 48.0,
            16.0,
            ui::MUTED,
        );
    }
    for factor in &conditions.factors {
        let good = factor.favors_attacker == me_attacking;
        let (sign, color) = if good { ("+", ui::GOOD) } else { ("-", ui::BAD) };
        let label = format!("{sign} {}", factor.label(me_attacking));
        let width = ui::draw_chip(d, &assets.info_font, Vector2::new(right.x + 24.0, y), &label, 15.0, color);
        d.draw_text_ex(&assets.info_font, &factor.effect(), Vector2::new(right.x + 34.0 + width, y + 4.0), 14.0, 1.0, ui::MUTED);
        y += 30.0;
    }

    // stats et bouton Jouer
    let stats_text = trf!(
        "Tries {}   PERFECT {}   Streak {} (best {})", practice.tries, practice.perfects, practice.streak, practice.best_streak ;
        "Essais {}   PARFAIT {}   Série {} (record {})", practice.tries, practice.perfects, practice.streak, practice.best_streak
    );
    let stats_width = assets.info_font.measure_text(&stats_text, 15.0, 1.0).x;
    d.draw_text_ex(&assets.info_font, &stats_text, Vector2::new(right.x + right.width / 2.0 - stats_width / 2.0, practice::play_rect().y - 30.0), 15.0, 1.0, ui::CREAM);
    ui::draw_button(d, assets, practice::play_rect(), tr("Play", "Jouer"), Skin::Wood, mouse, true);
    let key = settings::key_name(settings::key(Bind::Action));
    let or_text = tr("or", "ou");
    let or_width = assets.info_font.measure_text(or_text, 14.0, 1.0).x;
    let keycap_width = assets.info_font.measure_text(&key, 14.0, 1.0).x + 14.0;
    let total = or_width + 8.0 + keycap_width.max(22.4);
    let hint_x = right.x + right.width / 2.0 - total / 2.0;
    let hint_y = practice::play_rect().y + practice::play_rect().height + 6.0;
    d.draw_text_ex(&assets.info_font, or_text, Vector2::new(hint_x, hint_y + 3.0), 14.0, 1.0, ui::MUTED);
    ui::draw_keycap(d, &assets.info_font, Vector2::new(hint_x + or_width + 8.0, hint_y), &key, 14.0);

    ui::draw_button(d, assets, hud.btn_guide_back, tr("Guide", "Guide"), Skin::Wood, mouse, true);
    let _ = delta_time;
}

/// l'essai en cours : fond de combat, mon unité à gauche, l'adversaire à droite
fn draw_practice_duel(d: &mut RaylibDrawHandle, assets: &mut Assets, delta_time: f32, practice: &Practice) {
    draw_fullscreen_texture(d, &assets.combat_screen_background_texture);
    let (attacker_class, defender_class) = practice.duel_classes();
    let me_attacking = practice.role == Role::Attack;
    let (mine, theirs) = if me_attacking {
        (attacker_class, defender_class)
    } else {
        (defender_class, attacker_class)
    };
    // mêmes tailles et places que l'écran de combat
    let size = 500.0;
    let top = SCREEN_HEIGHT as f32 / 2.0 - size / 2.0;
    let center = SCREEN_WIDTH as f32 / 2.0;
    draw_idle_facing(d, &mut assets.animation_set_mut(mine).idle, center - size + 185.0, top, size, delta_time, false);
    draw_idle_facing(d, &mut assets.animation_set_mut(theirs).idle, center - 185.0, top, size, delta_time, true);
    let assets = &*assets;
    let role = if me_attacking { tr("attacks", "attaque") } else { tr("defends", "défend") };
    let title = format!("{} {}", mine.name(), role);
    let width = assets.alert_font.measure_text(&title, 32.0, 2.0).x;
    d.draw_text_ex(&assets.alert_font, &title, Vector2::new(SCREEN_WIDTH as f32 / 2.0 - width / 2.0 + 2.0, 22.0), 32.0, 2.0, Color::new(0, 0, 0, 160));
    d.draw_text_ex(&assets.alert_font, &title, Vector2::new(SCREEN_WIDTH as f32 / 2.0 - width / 2.0, 20.0), 32.0, 2.0, Color::WHITE);
    draw_combat_factors(d, assets, &practice.conditions(), me_attacking);
}

fn draw_practice_result(d: &mut RaylibDrawHandle, assets: &Assets, mouse: Vector2, practice: &Practice, result: TimingResult) {
    let me_attacking = practice.role == Role::Attack;
    let (word, color) = match result {
        TimingResult::Bad => (tr("BAD", "RATÉ"), ui::BAD),
        TimingResult::Good => (tr("GOOD", "BIEN"), Color::ORANGE),
        TimingResult::Perfect => (tr("PERFECT!", "PARFAIT !"), ui::GOOD),
    };
    let multiplier = result.to_multiplier(me_attacking);
    let detail = if me_attacking {
        trf!("damage dealt x{}", multiplier ; "dégâts infligés x{}", multiplier)
    } else {
        trf!("damage taken x{}", multiplier ; "dégâts subis x{}", multiplier)
    };
    let band = Rectangle::new(0.0, 160.0, SCREEN_WIDTH as f32, 250.0);
    d.draw_rectangle_rec(band, Color::new(0, 0, 0, 170));
    let width = assets.alert_font.measure_text(word, 84.0, 2.0).x;
    d.draw_text_ex(&assets.alert_font, word, Vector2::new(SCREEN_WIDTH as f32 / 2.0 - width / 2.0, 176.0), 84.0, 2.0, color);
    let mut line = detail;
    if practice.streak >= 2 {
        line += &trf!("   ·   PERFECT streak: {}", practice.streak ; "   ·   série PARFAITE : {}", practice.streak);
    }
    let width = assets.info_font.measure_text(&line, 20.0, 1.0).x;
    d.draw_text_ex(&assets.info_font, &line, Vector2::new(SCREEN_WIDTH as f32 / 2.0 - width / 2.0, 272.0), 20.0, 1.0, ui::CREAM);
    ui::draw_button(d, assets, practice::again_rect(), tr("Again", "Rejouer"), Skin::Wood, mouse, true);
    ui::draw_button(d, assets, practice::setup_rect(), tr("Change setup", "Modifier"), Skin::Wood, mouse, true);
}

/// sprite idle animé dans un carré de `size` (comme l'écran de combat), tourné vers la
/// gauche si `flip`
fn draw_idle_facing(
    d: &mut RaylibDrawHandle,
    animation: &mut Animation,
    x: f32,
    top_y: f32,
    size: f32,
    delta_time: f32,
    flip: bool,
) {
    animation.animation_update(delta_time);
    let mut frame = animation.animation_frame();
    if flip {
        frame.width = -frame.width;
    }
    d.draw_texture_pro(
        &animation.texture,
        frame,
        Rectangle::new(x, top_y, size, size),
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}
