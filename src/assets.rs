use crate::animation::Animation;
use raylib::prelude::*;

pub struct Assets {
    pub human_idle_animation: Animation,
    pub human_walking_animation: Animation,
    pub human_attack_animation: Animation,
    pub enemy_idle_animation: Animation,
    pub enemy_hurt_animation: Animation,
    pub enemy_dying_animation: Animation,
    pub mouse_normal_texture: Texture2D,
    pub mouse_hover_texture: Texture2D,
    pub mouse_click_texture: Texture2D,
    pub mouse_select_texture: Texture2D,
}

pub fn load_assets(rl: &mut RaylibHandle, thread: &RaylibThread) -> Assets {
    let human_idle_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Idle.png",
        )
        .unwrap();
    let human_walking_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Walk.png",
        )
        .unwrap();
    let human_attack_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Attact 1.png",
        )
        .unwrap();
    let enemy_idle_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith/Wraith-Idle.png",
        )
        .unwrap();
    let enemy_hurt_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith_split shadows/Wraith-Hurt.png",
        )
        .unwrap();
    let enemy_dying_texture = rl
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
        human_idle_animation: Animation {
            texture: human_idle_texture,
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
        human_walking_animation: Animation {
            texture: human_walking_texture,
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
        human_attack_animation: Animation {
            texture: human_attack_texture,
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
        enemy_idle_animation: Animation {
            texture: enemy_idle_texture,
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
        enemy_hurt_animation: Animation {
            texture: enemy_hurt_texture,
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
        enemy_dying_animation: Animation {
            texture: enemy_dying_texture,
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
