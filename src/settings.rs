//! réglages du joueur : langue, touches, confort. Gardés dans un petit fichier texte
//! (un `clé=valeur` par ligne) pour les retrouver à la prochaine partie.
//!
//! Tout est local : rien de ce qui est ici ne passe sur le réseau, deux joueurs en ligne
//! peuvent avoir des langues et des touches différentes.

use std::path::PathBuf;
use std::sync::RwLock;

use raylib::consts::KeyboardKey::{self, *};
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Lang {
    En,
    Fr,
}

/// une action qu'on peut lier à une touche. Deux groupes : la carte et les mini-jeux,
/// une même touche peut servir dans les deux (ex : Espace) sans conflit
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Bind {
    // carte
    Cancel,
    MapSpeed,
    EndTurn,
    Wait,
    // mini-jeux
    Action,
    Up,
    Down,
    Left,
    Right,
    Rune1,
    Rune2,
    Rune3,
    Rune4,
}

const BIND_COUNT: usize = 13;

impl Bind {
    pub const MAP: [Bind; 4] = [Bind::Cancel, Bind::MapSpeed, Bind::EndTurn, Bind::Wait];
    pub const DUEL: [Bind; 9] = [
        Bind::Action,
        Bind::Up,
        Bind::Down,
        Bind::Left,
        Bind::Right,
        Bind::Rune1,
        Bind::Rune2,
        Bind::Rune3,
        Bind::Rune4,
    ];
    pub const RUNES: [Bind; 4] = [Bind::Rune1, Bind::Rune2, Bind::Rune3, Bind::Rune4];

    /// les actions du même groupe ne peuvent pas partager une touche
    pub fn group(self) -> &'static [Bind] {
        if Bind::MAP.contains(&self) {
            &Bind::MAP
        } else {
            &Bind::DUEL
        }
    }

    /// nom dans le fichier de réglages (ne pas traduire, ne pas changer)
    fn id(self) -> &'static str {
        match self {
            Bind::Cancel => "cancel",
            Bind::MapSpeed => "map_speed",
            Bind::EndTurn => "end_turn",
            Bind::Wait => "wait",
            Bind::Action => "action",
            Bind::Up => "up",
            Bind::Down => "down",
            Bind::Left => "left",
            Bind::Right => "right",
            Bind::Rune1 => "rune1",
            Bind::Rune2 => "rune2",
            Bind::Rune3 => "rune3",
            Bind::Rune4 => "rune4",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Bind::Cancel => tr("Cancel action", "Annuler l'action"),
            Bind::MapSpeed => tr("Map speed x1 / x2", "Vitesse carte x1 / x2"),
            Bind::EndTurn => tr("End turn", "Fin du tour"),
            Bind::Wait => tr("Wait (unit)", "Attendre (unité)"),
            Bind::Action => tr("Strike / shoot", "Frapper / tirer"),
            Bind::Up => tr("Combo: up", "Combo : haut"),
            Bind::Down => tr("Combo: down", "Combo : bas"),
            Bind::Left => tr("Combo: left", "Combo : gauche"),
            Bind::Right => tr("Combo: right", "Combo : droite"),
            Bind::Rune1 => tr("Rune 1", "Rune 1"),
            Bind::Rune2 => tr("Rune 2", "Rune 2"),
            Bind::Rune3 => tr("Rune 3", "Rune 3"),
            Bind::Rune4 => tr("Rune 4", "Rune 4"),
        }
    }

    const ALL: [Bind; BIND_COUNT] = [
        Bind::Cancel,
        Bind::MapSpeed,
        Bind::EndTurn,
        Bind::Wait,
        Bind::Action,
        Bind::Up,
        Bind::Down,
        Bind::Left,
        Bind::Right,
        Bind::Rune1,
        Bind::Rune2,
        Bind::Rune3,
        Bind::Rune4,
    ];
}

const DEFAULT_KEYS: [KeyboardKey; BIND_COUNT] = [
    KEY_B,
    KEY_F,
    KEY_ENTER,
    KEY_SPACE,
    KEY_SPACE,
    // raylib lit la position physique : WASD = ZQSD sur un clavier AZERTY
    KEY_W,
    KEY_S,
    KEY_A,
    KEY_D,
    KEY_ONE,
    KEY_TWO,
    KEY_THREE,
    KEY_FOUR,
];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Settings {
    pub lang: Lang,
    keys: [KeyboardKey; BIND_COUNT],
    /// la carte démarre en x2
    pub fast_map: bool,
    /// lignes de la grille sur la carte
    pub grid_lines: bool,
    /// fenêtre d'aperçu du duel au survol d'un ennemi
    pub attack_preview: bool,
    /// bonus de placement (+12...) sur les cases d'attaque
    pub tile_bonuses: bool,
}

impl Settings {
    pub const DEFAULT: Settings = Settings {
        lang: Lang::En,
        keys: DEFAULT_KEYS,
        fast_map: false,
        grid_lines: false,
        attack_preview: true,
        tile_bonuses: true,
    };

    pub fn key(&self, bind: Bind) -> KeyboardKey {
        self.keys[bind as usize]
    }

    /// lie `bind` à `key`. Si une autre action du même groupe l'utilisait déjà, les
    /// deux échangent leurs touches : aucune action ne reste sans touche
    pub fn rebind(&mut self, bind: Bind, key: KeyboardKey) {
        let previous = self.key(bind);
        if let Some(&other) = bind
            .group()
            .iter()
            .find(|&&b| b != bind && self.key(b) == key)
        {
            self.keys[other as usize] = previous;
        }
        self.keys[bind as usize] = key;
    }

    pub fn reset_keys(&mut self) {
        self.keys = DEFAULT_KEYS;
    }

    fn to_text(self) -> String {
        let flag = |b: bool| if b { "1" } else { "0" };
        let mut text = format!(
            "lang={}\nfast_map={}\ngrid_lines={}\nattack_preview={}\ntile_bonuses={}\n",
            match self.lang {
                Lang::En => "en",
                Lang::Fr => "fr",
            },
            flag(self.fast_map),
            flag(self.grid_lines),
            flag(self.attack_preview),
            flag(self.tile_bonuses),
        );
        for bind in Bind::ALL {
            text += &format!("key.{}={}\n", bind.id(), self.key(bind) as i32);
        }
        text
    }

    /// lignes inconnues ou abîmées ignorées : on garde la valeur par défaut
    fn from_text(text: &str) -> Settings {
        let mut settings = Settings::DEFAULT;
        for line in text.lines() {
            let Some((name, value)) = line.split_once('=') else {
                continue;
            };
            let flag = value.trim() == "1";
            match name.trim() {
                "lang" => {
                    settings.lang = if value.trim() == "fr" {
                        Lang::Fr
                    } else {
                        Lang::En
                    }
                }
                "fast_map" => settings.fast_map = flag,
                "grid_lines" => settings.grid_lines = flag,
                "attack_preview" => settings.attack_preview = flag,
                "tile_bonuses" => settings.tile_bonuses = flag,
                name => {
                    let bind = Bind::ALL
                        .into_iter()
                        .find(|b| name.strip_prefix("key.") == Some(b.id()));
                    let key = value
                        .trim()
                        .parse()
                        .ok()
                        .and_then(raylib::core::input::key_from_i32);
                    if let (Some(bind), Some(key)) = (bind, key) {
                        settings.rebind(bind, key);
                    }
                }
            }
        }
        settings
    }
}

static SETTINGS: RwLock<Settings> = RwLock::new(Settings::DEFAULT);

/// réglages en cours (copie : ils sont petits)
pub fn get() -> Settings {
    *SETTINGS.read().unwrap()
}

/// change les réglages et les enregistre tout de suite
pub fn set(settings: Settings) {
    *SETTINGS.write().unwrap() = settings;
    save(settings);
}

/// change les réglages sans les enregistrer (visite de vérification, devshots.rs)
#[cfg(debug_assertions)]
pub fn set_unsaved(settings: Settings) {
    *SETTINGS.write().unwrap() = settings;
}

/// au lancement : relit le fichier s'il existe
pub fn load() {
    if let Some(text) = file().and_then(|path| std::fs::read_to_string(path).ok()) {
        *SETTINGS.write().unwrap() = Settings::from_text(&text);
    }
}

fn save(settings: Settings) {
    let Some(path) = file() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, settings.to_text());
}

/// dossier de config habituel de chaque OS (pas à côté du jeu : la mise à jour
/// automatique remplace ce dossier-là)
fn file() -> Option<PathBuf> {
    let dir = if cfg!(windows) {
        PathBuf::from(std::env::var_os("APPDATA")?)
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support")
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".config")))?
    };
    Some(dir.join("ashes-bones").join("settings.cfg"))
}

// langue ------------------------------------------------------------------------

pub fn is_fr() -> bool {
    get().lang == Lang::Fr
}

/// le texte dans la langue choisie
pub fn tr(en: &'static str, fr: &'static str) -> &'static str {
    if is_fr() { fr } else { en }
}

/// comme `format!`, une version par langue : trf!("{} games", n ; "{} parties", n)
#[macro_export]
macro_rules! trf {
    ($en:literal $(, $en_arg:expr)* ; $fr:literal $(, $fr_arg:expr)* $(,)?) => {
        if $crate::settings::is_fr() {
            format!($fr $(, $fr_arg)*)
        } else {
            format!($en $(, $en_arg)*)
        }
    };
}

// touches ------------------------------------------------------------------------

pub fn key(bind: Bind) -> KeyboardKey {
    get().key(bind)
}

pub fn pressed(rl: &RaylibHandle, bind: Bind) -> bool {
    rl.is_key_pressed(key(bind))
}

/// nom affiché d'une touche, selon la disposition du clavier (Z sur AZERTY pour KEY_W)
pub fn key_name(key: KeyboardKey) -> String {
    let named = match key {
        KEY_SPACE => tr("Space", "Espace"),
        KEY_ENTER => tr("Enter", "Entrée"),
        KEY_KP_ENTER => tr("Num Enter", "Entrée num"),
        KEY_TAB => "Tab",
        KEY_BACKSPACE => tr("Backspace", "Retour"),
        KEY_ESCAPE => tr("Esc", "Échap"),
        KEY_UP => "↑",
        KEY_DOWN => "↓",
        KEY_LEFT => "←",
        KEY_RIGHT => "→",
        KEY_LEFT_SHIFT | KEY_RIGHT_SHIFT => tr("Shift", "Maj"),
        KEY_LEFT_CONTROL | KEY_RIGHT_CONTROL => "Ctrl",
        KEY_LEFT_ALT | KEY_RIGHT_ALT => "Alt",
        KEY_LEFT_SUPER | KEY_RIGHT_SUPER => "Cmd",
        KEY_DELETE => tr("Delete", "Suppr"),
        KEY_INSERT => tr("Insert", "Inser"),
        KEY_HOME => tr("Home", "Début"),
        KEY_END => tr("End", "Fin"),
        KEY_PAGE_UP => tr("Page up", "Page haut"),
        KEY_PAGE_DOWN => tr("Page down", "Page bas"),
        KEY_CAPS_LOCK => tr("Caps lock", "Verr. maj"),
        _ => "",
    };
    if !named.is_empty() {
        return named.to_string();
    }
    let code = key as i32;
    // rangée des chiffres : le chiffre est imprimé sur la touche, même en AZERTY (& 1)
    if (KEY_ZERO as i32..=KEY_NINE as i32).contains(&code) {
        return ((b'0' + (code - KEY_ZERO as i32) as u8) as char).to_string();
    }
    if (KEY_KP_0 as i32..=KEY_KP_9 as i32).contains(&code) {
        return format!("Num {}", code - KEY_KP_0 as i32);
    }
    if (KEY_F1 as i32..=KEY_F12 as i32).contains(&code) {
        return format!("F{}", code - KEY_F1 as i32 + 1);
    }
    // lettres et ponctuation : nom selon la disposition du clavier
    let layout_name = unsafe {
        let name = raylib::ffi::GetKeyName(code);
        (!name.is_null()).then(|| std::ffi::CStr::from_ptr(name).to_string_lossy().to_string())
    };
    match layout_name {
        Some(name) if !name.trim().is_empty() => name.to_uppercase(),
        _ => format!("#{code}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebind_swaps_inside_a_group() {
        let mut s = Settings::DEFAULT;
        // F est déjà la vitesse de la carte : elle récupère l'ancienne touche d'Annuler
        s.rebind(Bind::Cancel, KEY_F);
        assert_eq!(s.key(Bind::Cancel), KEY_F);
        assert_eq!(s.key(Bind::MapSpeed), KEY_B);
    }

    #[test]
    fn same_key_allowed_in_both_groups() {
        let mut s = Settings::DEFAULT;
        s.rebind(Bind::Action, KEY_B);
        assert_eq!(s.key(Bind::Action), KEY_B);
        assert_eq!(s.key(Bind::Cancel), KEY_B);
    }

    #[test]
    fn file_round_trip() {
        let mut s = Settings::DEFAULT;
        s.lang = Lang::Fr;
        s.fast_map = true;
        s.tile_bonuses = false;
        s.rebind(Bind::Up, KEY_I);
        assert_eq!(Settings::from_text(&s.to_text()), s);
    }

    #[test]
    fn broken_file_keeps_defaults() {
        let s = Settings::from_text("lang=fr\nkey.up=abc\nnonsense\nkey.unknown=65\n");
        assert_eq!(s.lang, Lang::Fr);
        assert_eq!(s.key(Bind::Up), KEY_W);
    }
}
