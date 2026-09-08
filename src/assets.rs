use crate::animation::Animation;
use raylib::prelude::*;

pub struct AnimationSet {
    pub idle: Animation,
    pub walk: Animation,
    pub attack: Animation,
    pub attack_effect: Animation,
    pub hurt: Animation,
    pub die: Animation,
}

pub struct Assets {
    pub soldier: AnimationSet,
    pub wraith: AnimationSet,
    pub mouse_normal_texture: Texture2D,
    pub mouse_hover_texture: Texture2D,
    pub mouse_click_texture: Texture2D,
    pub mouse_select_texture: Texture2D,
    pub combat_screen_background_texture: Texture2D,
    pub hud_font: Font,
}

pub fn load_assets(rl: &mut RaylibHandle, thread: &RaylibThread) -> Assets {
    let soldier_idle_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Idle.png",
        )
        .unwrap();
    let soldier_walking_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Walk.png",
        )
        .unwrap();
    let soldier_attack_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Attact 1.png",
        )
        .unwrap();
    let soldier_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier effects/human_soldier-Attact 1 effect.png",
        )
        .unwrap();
    let soldier_hurt_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Hurt.png",
        )
        .unwrap();
    let soldier_die_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Die.png",
        )
        .unwrap();
    let wraith_idle_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith/Wraith-Idle.png",
        )
        .unwrap();
    let wraith_walking_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith/Wraith-Walk.png",
        )
        .unwrap();
    let wraith_attack_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith/Wraith-Attack 2.png",
        )
        .unwrap();
    let wraith_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith_Effects/Wraith Effects-Attack 2.png",
        )
        .unwrap();
    let wraith_hurt_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith_split shadows/Wraith-Hurt.png",
        )
        .unwrap();
    let wraith_dying_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith_split shadows/Wraith-Die.png",
        )
        .unwrap();
    let mouse_normal_texture = rl
        .load_texture(thread, "assets/cursors/PNG/01.png")
        .unwrap();
    let mouse_hover_texture = rl
        .load_texture(thread, "assets/cursors/PNG/10.png")
        .unwrap();
    let mouse_click_texture = rl
        .load_texture(thread, "assets/cursors/PNG/13.png")
        .unwrap();
    let mouse_select_texture = rl
        .load_texture(thread, "assets/cursors/selector_frame_v2.png")
        .unwrap();
    let combat_screen_background_texture = rl
        .load_texture(thread, "assets/backgrounds/plains2.png")
        .unwrap();
    let hud_font = rl
        .load_font(thread, "assets/fonts/BerkshireSwash-Regular.ttf")
        .unwrap();

    Assets {
        soldier: AnimationSet {
            idle: Animation {
                texture: soldier_idle_texture,
                frame_width: 130,
                frame_height: 100,
                frames_per_row: 7,
                first: 0,
                last: 6,
                current: 0,
                speed: 4.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            walk: Animation {
                texture: soldier_walking_texture,
                frame_width: 130,
                frame_height: 100,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            attack: Animation {
                texture: soldier_attack_texture,
                frame_width: 130,
                frame_height: 100,
                frames_per_row: 7,
                first: 0,
                last: 6,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            attack_effect: Animation {
                texture: soldier_attack_effect_texture,
                frame_width: 130,
                frame_height: 100,
                frames_per_row: 7,
                first: 0,
                last: 6,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            hurt: Animation {
                texture: soldier_hurt_texture,
                frame_width: 130,
                frame_height: 100,
                frames_per_row: 6,
                first: 0,
                last: 5,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            die: Animation {
                texture: soldier_die_texture,
                frame_width: 130,
                frame_height: 100,
                frames_per_row: 7,
                first: 0,
                last: 6,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
        },
        wraith: AnimationSet {
            idle: Animation {
                texture: wraith_idle_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            walk: Animation {
                texture: wraith_walking_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            attack: Animation {
                texture: wraith_attack_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            attack_effect: Animation {
                texture: wraith_attack_effect_texture,
                frame_width: 130,
                frame_height: 100,
                frames_per_row: 6,
                first: 0,
                last: 5,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            hurt: Animation {
                texture: wraith_hurt_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 6,
                first: 0,
                last: 5,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            die: Animation {
                texture: wraith_dying_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
        },
        mouse_normal_texture,
        mouse_hover_texture,
        mouse_click_texture,
        mouse_select_texture,
        combat_screen_background_texture,
        hud_font,
    }
}
