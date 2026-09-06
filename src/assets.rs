use crate::animation::Animation;
use raylib::prelude::*;

pub struct Assets {
    pub soldier_idle_animation: Animation,
    pub soldier_walking_animation: Animation,
    pub soldier_attack_animation: Animation,
    pub soldier_attack_effect_animation: Animation,
    pub wraith_idle_animation: Animation,
    pub wraith_walking_animation: Animation,
    pub wraith_hurt_animation: Animation,
    pub wraith_dying_animation: Animation,
    pub mouse_normal_texture: Texture2D,
    pub mouse_hover_texture: Texture2D,
    pub mouse_click_texture: Texture2D,
    pub mouse_select_texture: Texture2D,
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

    Assets {
        soldier_idle_animation: Animation {
            texture: soldier_idle_texture,
            frame_width: 130,
            frame_height: 100,
            frames_per_row: 7,
            first: 0,
            last: 6,
            current: 0,
            speed: 8.0,
            duration_left: 0.1,
            finished: false,
            looping: true,
        },
        soldier_walking_animation: Animation {
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
        soldier_attack_animation: Animation {
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
        soldier_attack_effect_animation: Animation {
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
        wraith_idle_animation: Animation {
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
        wraith_walking_animation: Animation {
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
        wraith_hurt_animation: Animation {
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
        wraith_dying_animation: Animation {
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
        mouse_normal_texture,
        mouse_hover_texture,
        mouse_click_texture,
        mouse_select_texture,
    }
}
