use crate::animation::Animation;
use crate::cursor::CursorType;
use crate::unit::UnitState;
use raylib::prelude::*;

pub struct AnimationSet {
    pub idle: Animation,
    pub walk: Animation,
    pub attack: Animation,
    pub attack_effect: Animation,
    pub hurt: Animation,
    pub die: Animation,
}

impl AnimationSet {
    pub fn for_state_mut(&mut self, state: UnitState) -> &mut Animation {
        match state {
            UnitState::Idle => &mut self.idle,
            UnitState::Walking => &mut self.walk,
            UnitState::Attacking => &mut self.attack,
            UnitState::ChoosingPosition => &mut self.idle,
            UnitState::CombatEntering => &mut self.walk,
            UnitState::Hurt => &mut self.hurt,
            UnitState::Dying => &mut self.die,
            UnitState::Dead => &mut self.idle,
            UnitState::MiniGame => &mut self.idle,
        }
    }
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
    pub alert_font: Font,
    pub btn_end_turn: Texture2D,
    pub btn_wait: Texture2D,
    pub btn_exit: Texture2D,
    pub btn_retry: Texture2D,
    pub btn_back: Texture2D,
    pub banner_your_turn: Texture2D,
    pub banner_enemy_turn: Texture2D,
    pub title_screen_background_texture_1: Texture2D,
    pub title_screen_background_texture_2: Texture2D,
    pub title_screen_background_texture_3: Texture2D,
    pub title_screen_background_texture_4: Texture2D,
    pub title_screen_background_texture_5: Texture2D,
    pub btn_exit_title_screen: Texture2D,
    pub btn_play_title_screen: Texture2D,
    pub btn_settings_title_screen: Texture2D,
    pub title: Texture2D,
}

impl Assets {
    pub fn cursor_texture(&self, cursor_type: CursorType) -> &Texture2D {
        match cursor_type {
            CursorType::Normal => &self.mouse_normal_texture,
            CursorType::Hover => &self.mouse_hover_texture,
            CursorType::Click => &self.mouse_click_texture,
        }
    }
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
    let alert_font = rl.load_font(thread, "assets/fonts/ThaleahFat.ttf").unwrap();
    let btn_end_turn = rl
        .load_texture(thread, "assets/turnbased-ui/btn_end_turn_normal.png")
        .unwrap();
    let btn_retry = rl
        .load_texture(thread, "assets/turnbased-ui/btn_retry_normal.png")
        .unwrap();
    let btn_exit = rl
        .load_texture(thread, "assets/turnbased-ui/btn_exit_normal.png")
        .unwrap();
    let btn_back = rl
        .load_texture(thread, "assets/turnbased-ui/btn_back_normal.png")
        .unwrap();
    let btn_wait = rl
        .load_texture(thread, "assets/turnbased-ui/btn_wait_normal.png")
        .unwrap();
    let banner_your_turn = rl
        .load_texture(thread, "assets/turnbased-ui/banner_your_turn.png")
        .unwrap();
    let banner_enemy_turn = rl
        .load_texture(thread, "assets/turnbased-ui/banner_enemy_phase.png")
        .unwrap();
    let title_screen_background_texture_1 = rl
        .load_texture(thread, "assets/backgrounds/Pref/1.png")
        .unwrap();
    let title_screen_background_texture_2 = rl
        .load_texture(thread, "assets/backgrounds/Pref/2.png")
        .unwrap();
    let title_screen_background_texture_3 = rl
        .load_texture(thread, "assets/backgrounds/Pref/3.png")
        .unwrap();
    let title_screen_background_texture_4 = rl
        .load_texture(thread, "assets/backgrounds/Pref/4.png")
        .unwrap();
    let title_screen_background_texture_5 = rl
        .load_texture(thread, "assets/backgrounds/Pref/5.png")
        .unwrap();
    let btn_exit_title_screen = rl
        .load_texture(thread, "assets/menu-ui/btn_exit_normal.png")
        .unwrap();
    let btn_play_title_screen = rl
        .load_texture(thread, "assets/menu-ui/btn_play_normal.png")
        .unwrap();
    let btn_settings_title_screen = rl
        .load_texture(thread, "assets/menu-ui/btn_settings_normal.png")
        .unwrap();
    let title = rl
        .load_texture(thread, "assets/menu-ui/title_ashes_bones.png")
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
                frames_per_row: 10,
                first: 0,
                last: 9,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            attack_effect: Animation {
                texture: wraith_attack_effect_texture,
                frame_width: 162,
                frame_height: 162,
                frames_per_row: 10,
                first: 0,
                last: 9,
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
        alert_font,
        btn_end_turn,
        btn_retry,
        btn_back,
        btn_exit,
        btn_wait,
        banner_your_turn,
        banner_enemy_turn,
        title_screen_background_texture_1,
        title_screen_background_texture_2,
        title_screen_background_texture_3,
        title_screen_background_texture_4,
        title_screen_background_texture_5,
        btn_exit_title_screen,
        btn_play_title_screen,
        btn_settings_title_screen,
        title,
    }
}
