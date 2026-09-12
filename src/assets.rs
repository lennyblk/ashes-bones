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
    pub assassin: AnimationSet,
    pub cavalry: AnimationSet,
    pub longbowman: AnimationSet,
    pub mage: AnimationSet,
    pub priest: AnimationSet,
    pub priest_aura: Animation,
    pub priest_aura_effect: Animation,
    pub priest_heal: Animation,
    pub priest_heal_effect: Animation,
    pub banshee: AnimationSet,
    pub blood_knight: AnimationSet,
    pub ghoul: AnimationSet,
    pub skeleton: AnimationSet,
    pub necromancer: AnimationSet,
    pub necromancer_attack2: Animation,
    pub necromancer_attack2_effect: Animation,
    pub necromancer_attack3: Animation,
    pub necromancer_attack3_effect: Animation,
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

    let assassin_idle_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human assassin/Human assassin/human_assassin_idle.png",
        )
        .unwrap();
    let assassin_walking_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human assassin/Human assassin/human_assassin_walk.png",
        )
        .unwrap();
    let assassin_attack_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human assassin/Human assassin/human_assassin_attack 1.png",
        )
        .unwrap();
    let assassin_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human assassin/Human assassin effects/human_assassin_attack 1.png",
        )
        .unwrap();
    let assassin_hurt_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human assassin/Human assassin/human_assassin_hurt.png",
        )
        .unwrap();
    let assassin_die_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human assassin/Human assassin/human_assassin_die.png",
        )
        .unwrap();

    let cavalry_idle_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human cavalry/Human cavalry/human_cavalry_idle.png",
        )
        .unwrap();
    let cavalry_walking_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human cavalry/Human cavalry/human_cavalry_walk 1.png",
        )
        .unwrap();
    let cavalry_attack_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human cavalry/Human cavalry/human_cavalry_attack 2.png",
        )
        .unwrap();
    let cavalry_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human cavalry/Human cavalry effects/human_cavalry_attack 2 effect.png",
        )
        .unwrap();
    let cavalry_hurt_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human cavalry/Human cavalry/human_cavalry_hurt.png",
        )
        .unwrap();
    let cavalry_die_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human cavalry/Human cavalry/human_cavalry_die.png",
        )
        .unwrap();

    let longbowman_idle_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human longbowman/Human longbowman/human_longbowman_idle.png",
        )
        .unwrap();
    let longbowman_walking_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human longbowman/Human longbowman/human_longbowman_walk.png",
        )
        .unwrap();
    let longbowman_attack_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human longbowman/Human longbowman/human_longbowman_attack 2.png",
        )
        .unwrap();
    let longbowman_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human longbowman/Human longbowman effects/human_longbowman_attack 2 effect.png",
        )
        .unwrap();
    let longbowman_hurt_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human longbowman/Human longbowman/human_longbowman_hurt.png",
        )
        .unwrap();
    let longbowman_die_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human longbowman/Human longbowman/human_longbowman_die.png",
        )
        .unwrap();

    let mage_idle_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human mage/Human mage/human_mage-Idle.png",
        )
        .unwrap();
    let mage_walking_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human mage/Human mage/human_mage-Walk.png",
        )
        .unwrap();
    let mage_attack_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human mage/Human mage/human_mage-Attack 1.png",
        )
        .unwrap();
    let mage_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human mage/Human mage effects/human_mage_attack 1.png",
        )
        .unwrap();
    let mage_hurt_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human mage/Human mage/human_mage-Hurt.png",
        )
        .unwrap();
    let mage_die_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human mage/Human mage/human_mage-Die.png",
        )
        .unwrap();

    let priest_idle_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest/human_priest-Idle.png",
        )
        .unwrap();
    let priest_walking_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest/human_priest-Walk.png",
        )
        .unwrap();
    let priest_attack_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest/human_priest-Attack.png",
        )
        .unwrap();
    let priest_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest effects/human_priest-Attack effect.png",
        )
        .unwrap();
    let priest_hurt_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest/human_priest-Hurt.png",
        )
        .unwrap();
    let priest_die_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest/human_priest-Die.png",
        )
        .unwrap();
    let priest_aura_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest/human_priest-Aura.png",
        )
        .unwrap();
    let priest_aura_effect_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest effects/human_priest-Aura effect.png",
        )
        .unwrap();
    let priest_heal_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest/human_priest-Heal.png",
        )
        .unwrap();
    let priest_heal_effect_texture = rl
        .load_texture(
            thread,
            "assets/humanChar/Human priest/Human priest effects/human_priest-Heal effect.png",
        )
        .unwrap();

    let banshee_idle_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Banshee 32x32/Undead Banshee/Banshee-Idle.png",
        )
        .unwrap();
    let banshee_walking_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Banshee 32x32/Undead Banshee/Banshee-Walk.png",
        )
        .unwrap();
    let banshee_attack_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Banshee 32x32/Undead Banshee/Banshee-Attack 2.png",
        )
        .unwrap();
    let banshee_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Banshee 32x32/Undead Bashee_Effects/Banshee Effects-Attack 2.png",
        )
        .unwrap();
    let banshee_hurt_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Banshee 32x32/Undead Bashee_split shadows/Banshee-Hurt.png",
        )
        .unwrap();
    let banshee_die_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Banshee 32x32/Undead Bashee_split shadows/Banshee-Die.png",
        )
        .unwrap();

    let blood_knight_idle_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Blood Knight 32x32/Undead Blood Knight/Blood Knight-Idle.png",
        )
        .unwrap();
    let blood_knight_walking_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Blood Knight 32x32/Undead Blood Knight/Blood Knight-Walk.png",
        )
        .unwrap();
    let blood_knight_attack_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Blood Knight 32x32/Undead Blood Knight/Blood Knight-Attack 2.png",
        )
        .unwrap();
    let blood_knight_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Blood Knight 32x32/Undead Blood Knight_Effects/Blood Knight Effects-Attack 2.png",
        )
        .unwrap();
    let blood_knight_hurt_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Blood Knight 32x32/Undead Blood Knight_split shadows/Blood Knight-Hurt.png",
        )
        .unwrap();
    let blood_knight_die_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Blood Knight 32x32/Undead Blood Knight_split shadows/Blood Knight-Die.png",
        )
        .unwrap();

    let ghoul_idle_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Ghoul 32x32/Undead Ghoul/Ghoul-Idle.png",
        )
        .unwrap();
    let ghoul_walking_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Ghoul 32x32/Undead Ghoul/Ghoul-Walk.png",
        )
        .unwrap();
    let ghoul_attack_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Ghoul 32x32/Undead Ghoul/Ghoul-Attack 2.png",
        )
        .unwrap();
    let ghoul_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Ghoul 32x32/Undead Ghoul_Effects/Ghoul Effects-Attack 2.png",
        )
        .unwrap();
    let ghoul_hurt_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Ghoul 32x32/Undead Ghoul_split shadows/Ghoul-Hurt.png",
        )
        .unwrap();
    let ghoul_die_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Ghoul 32x32/Undead Ghoul_split shadows/Ghoul-Die.png",
        )
        .unwrap();

    let skeleton_idle_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Skeleton 32x32/Undead Skeleton/Skeleton-Idle.png",
        )
        .unwrap();
    let skeleton_walking_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Skeleton 32x32/Undead Skeleton/Skeleton-Walk.png",
        )
        .unwrap();
    let skeleton_attack_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Skeleton 32x32/Undead Skeleton/Skeleton-Attack 2.png",
        )
        .unwrap();
    let skeleton_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Skeleton 32x32/Undead Skeleton_Effects/Skeleton Effects-Attack 2.png",
        )
        .unwrap();
    let skeleton_hurt_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Skeleton 32x32/Undead Skeleton_split shadows/Skeleton-Hurt.png",
        )
        .unwrap();
    let skeleton_die_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Skeleton 32x32/Undead Skeleton_split shadows/Skeleton-Die.png",
        )
        .unwrap();

    let necromancer_idle_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer/Necromancer-Idle.png",
        )
        .unwrap();
    let necromancer_walking_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer/Necromancer-Walk.png",
        )
        .unwrap();
    let necromancer_attack_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer/Necromancer-Attack 1.png",
        )
        .unwrap();
    let necromancer_attack_effect_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer_Effects/Necromancer Effects-Attack 1.png",
        )
        .unwrap();
    let necromancer_hurt_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer/Necromancer-Hurt.png",
        )
        .unwrap();
    let necromancer_die_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer/Necromancer-Die.png",
        )
        .unwrap();
    let necromancer_attack2_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer/Necromancer-Attack 2.png",
        )
        .unwrap();
    let necromancer_attack2_effect_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer_Effects/Necromancer Effects-Attack 2.png",
        )
        .unwrap();
    let necromancer_attack3_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer/Necromancer-Attack 3.png",
        )
        .unwrap();
    let necromancer_attack3_effect_texture = rl
        .load_texture(
            thread,
            "assets/undeadChar/Undead Necromancer 32x32/Undead Necromancer_Effects/Necromancer Effects-Attack 3.png",
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
        .load_font_ex(thread, "assets/fonts/BerkshireSwash-Regular.ttf", 96, None)
        .unwrap();
    hud_font
        .texture()
        .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
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
        assassin: AnimationSet {
            idle: Animation {
                texture: assassin_idle_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: assassin_walking_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: assassin_attack_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: assassin_attack_effect_texture,
                frame_width: 130,
                frame_height: 102,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            hurt: Animation {
                texture: assassin_hurt_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: assassin_die_texture,
                frame_width: 130,
                frame_height: 102,
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
        cavalry: AnimationSet {
            idle: Animation {
                texture: cavalry_idle_texture,
                frame_width: 130,
                frame_height: 102,
                frames_per_row: 6,
                first: 0,
                last: 5,
                current: 0,
                speed: 4.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            walk: Animation {
                texture: cavalry_walking_texture,
                frame_width: 130,
                frame_height: 102,
                frames_per_row: 7,
                first: 0,
                last: 6,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            attack: Animation {
                texture: cavalry_attack_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: cavalry_attack_effect_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: cavalry_hurt_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: cavalry_die_texture,
                frame_width: 130,
                frame_height: 102,
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
        longbowman: AnimationSet {
            idle: Animation {
                texture: longbowman_idle_texture,
                frame_width: 130,
                frame_height: 102,
                frames_per_row: 6,
                first: 0,
                last: 5,
                current: 0,
                speed: 4.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            walk: Animation {
                texture: longbowman_walking_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: longbowman_attack_texture,
                frame_width: 130,
                frame_height: 102,
                frames_per_row: 9,
                first: 0,
                last: 8,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            attack_effect: Animation {
                texture: longbowman_attack_effect_texture,
                frame_width: 130,
                frame_height: 102,
                frames_per_row: 9,
                first: 0,
                last: 8,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            hurt: Animation {
                texture: longbowman_hurt_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: longbowman_die_texture,
                frame_width: 130,
                frame_height: 102,
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
        mage: AnimationSet {
            idle: Animation {
                texture: mage_idle_texture,
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
                texture: mage_walking_texture,
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
                texture: mage_attack_texture,
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
                texture: mage_attack_effect_texture,
                frame_width: 130,
                frame_height: 102,
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
                texture: mage_hurt_texture,
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
                texture: mage_die_texture,
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
        },
        priest: AnimationSet {
            idle: Animation {
                texture: priest_idle_texture,
                frame_width: 130,
                frame_height: 100,
                frames_per_row: 6,
                first: 0,
                last: 5,
                current: 0,
                speed: 4.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            walk: Animation {
                texture: priest_walking_texture,
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
                texture: priest_attack_texture,
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
                texture: priest_attack_effect_texture,
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
                texture: priest_hurt_texture,
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
                texture: priest_die_texture,
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
        priest_aura: Animation {
            texture: priest_aura_texture,
            frame_width: 130,
            frame_height: 100,
            frames_per_row: 9,
            first: 0,
            last: 8,
            current: 0,
            speed: 8.0,
            duration_left: 0.1,
            finished: false,
            looping: false,
        },
        priest_aura_effect: Animation {
            texture: priest_aura_effect_texture,
            frame_width: 130,
            frame_height: 100,
            frames_per_row: 9,
            first: 0,
            last: 8,
            current: 0,
            speed: 8.0,
            duration_left: 0.1,
            finished: false,
            looping: false,
        },
        priest_heal: Animation {
            texture: priest_heal_texture,
            frame_width: 130,
            frame_height: 100,
            frames_per_row: 8,
            first: 0,
            last: 7,
            current: 0,
            speed: 8.0,
            duration_left: 0.1,
            finished: false,
            looping: false,
        },
        priest_heal_effect: Animation {
            texture: priest_heal_effect_texture,
            frame_width: 130,
            frame_height: 100,
            frames_per_row: 8,
            first: 0,
            last: 7,
            current: 0,
            speed: 8.0,
            duration_left: 0.1,
            finished: false,
            looping: false,
        },
        banshee: AnimationSet {
            idle: Animation {
                texture: banshee_idle_texture,
                frame_width: 160,
                frame_height: 160,
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
                texture: banshee_walking_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 7,
                first: 0,
                last: 6,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            attack: Animation {
                texture: banshee_attack_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 13,
                first: 0,
                last: 12,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            attack_effect: Animation {
                texture: banshee_attack_effect_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 13,
                first: 0,
                last: 12,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            hurt: Animation {
                texture: banshee_hurt_texture,
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
                texture: banshee_die_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 9,
                first: 0,
                last: 8,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
        },
        blood_knight: AnimationSet {
            idle: Animation {
                texture: blood_knight_idle_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 6,
                first: 0,
                last: 5,
                current: 0,
                speed: 4.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            walk: Animation {
                texture: blood_knight_walking_texture,
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
                texture: blood_knight_attack_texture,
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
                texture: blood_knight_attack_effect_texture,
                frame_width: 162,
                frame_height: 162,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            hurt: Animation {
                texture: blood_knight_hurt_texture,
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
                texture: blood_knight_die_texture,
                frame_width: 160,
                frame_height: 160,
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
        ghoul: AnimationSet {
            idle: Animation {
                texture: ghoul_idle_texture,
                frame_width: 160,
                frame_height: 160,
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
                texture: ghoul_walking_texture,
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
                texture: ghoul_attack_texture,
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
                texture: ghoul_attack_effect_texture,
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
                texture: ghoul_hurt_texture,
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
                texture: ghoul_die_texture,
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
        skeleton: AnimationSet {
            idle: Animation {
                texture: skeleton_idle_texture,
                frame_width: 160,
                frame_height: 160,
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
                texture: skeleton_walking_texture,
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
                texture: skeleton_attack_texture,
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
                texture: skeleton_attack_effect_texture,
                frame_width: 162,
                frame_height: 162,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            hurt: Animation {
                texture: skeleton_hurt_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 7,
                first: 0,
                last: 6,
                current: 0,
                speed: 8.0,
                duration_left: 0.1,
                finished: false,
                looping: false,
            },
            die: Animation {
                texture: skeleton_die_texture,
                frame_width: 160,
                frame_height: 160,
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
        necromancer: AnimationSet {
            idle: Animation {
                texture: necromancer_idle_texture,
                frame_width: 160,
                frame_height: 160,
                frames_per_row: 8,
                first: 0,
                last: 7,
                current: 0,
                speed: 4.0,
                duration_left: 0.1,
                finished: false,
                looping: true,
            },
            walk: Animation {
                texture: necromancer_walking_texture,
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
                texture: necromancer_attack_texture,
                frame_width: 160,
                frame_height: 160,
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
                texture: necromancer_attack_effect_texture,
                frame_width: 162,
                frame_height: 162,
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
                texture: necromancer_hurt_texture,
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
                texture: necromancer_die_texture,
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
        necromancer_attack2: Animation {
            texture: necromancer_attack2_texture,
            frame_width: 160,
            frame_height: 160,
            frames_per_row: 9,
            first: 0,
            last: 8,
            current: 0,
            speed: 8.0,
            duration_left: 0.1,
            finished: false,
            looping: false,
        },
        necromancer_attack2_effect: Animation {
            texture: necromancer_attack2_effect_texture,
            frame_width: 162,
            frame_height: 162,
            frames_per_row: 9,
            first: 0,
            last: 8,
            current: 0,
            speed: 8.0,
            duration_left: 0.1,
            finished: false,
            looping: false,
        },
        necromancer_attack3: Animation {
            texture: necromancer_attack3_texture,
            frame_width: 160,
            frame_height: 160,
            frames_per_row: 12,
            first: 0,
            last: 11,
            current: 0,
            speed: 8.0,
            duration_left: 0.1,
            finished: false,
            looping: false,
        },
        necromancer_attack3_effect: Animation {
            texture: necromancer_attack3_effect_texture,
            frame_width: 162,
            frame_height: 162,
            frames_per_row: 12,
            first: 0,
            last: 11,
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
