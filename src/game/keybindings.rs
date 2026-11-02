use minifb::Key;
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct KeyBindings {
    pub forward: u32,
    pub backward: u32,
    pub left: u32,
    pub right: u32,
    pub sprint: u32,
    pub crouch: u32,
    pub jump: u32,
    pub rocket_jump: u32,
    pub fire_bullet: u32,
    pub fire_arrow: u32,
    pub interact: u32,
    pub godmode: u32,
    pub toggle_aim: u32,
    pub enemy_lock: u32,
    pub aim_up: u32,
    pub aim_down: u32,
    pub turn_left: u32,
    pub turn_right: u32,
    pub menu_escape: u32,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            forward: Key::W as u32,
            backward: Key::S as u32,
            left: Key::A as u32,
            right: Key::D as u32,
            sprint: Key::LeftShift as u32,
            crouch: Key::C as u32,
            jump: Key::Space as u32,
            rocket_jump: Key::R as u32,
            fire_bullet: Key::RightCtrl as u32,
            fire_arrow: Key::RightShift as u32,
            interact: Key::F as u32,
            godmode: Key::G as u32,
            toggle_aim: Key::E as u32,
            enemy_lock: Key::L as u32,
            aim_up: Key::Up as u32,
            aim_down: Key::Down as u32,
            turn_left: Key::Left as u32,
            turn_right: Key::Right as u32,
            menu_escape: Key::Escape as u32,
        }
    }
}

impl KeyBindings {
    pub fn load() -> Self {
        if let Ok(data) = fs::read_to_string("keybindings.json") {
            if let Ok(bindings) = serde_json::from_str(&data) {
                return bindings;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(data) = serde_json::to_string(self) {
            let _ = fs::write("keybindings.json", data);
        }
    }

    pub fn to_key(code: u32) -> Key {
        unsafe { std::mem::transmute(code as u8) }
    }
}