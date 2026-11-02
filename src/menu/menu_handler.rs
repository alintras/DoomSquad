//! The Menu and User Interface system.
//!
//! This module manages all the non-gameplay screens, such as the Main Menu,
//! the Pause Screen, and the Game Over screen. It handles drawing buttons and text,
//! processing mouse clicks, and taking care of saving and loading the player's progress.

use font8x8::{BASIC_FONTS, UnicodeFonts};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};
use std::fs;

use crate::audio::audio_handler::Audio;
use crate::game::gamestate::Game;
use crate::render::RendererData;
use crate::game::keybindings::KeyBindings;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;

const COL_BG_MAIN: u32 = 0x111111;
const COL_BG_GAMEOVER: u32 = 0x330000;
const COL_BG_PAUSED: u32 = 0x002244;
const COL_TITLE_RED: u32 = 0xCC0000;
const COL_TITLE_BLUE: u32 = 0x44AAFF;
const COL_BTN_NORMAL: u32 = 0x444444;
const COL_BTN_HOVER: u32 = 0xAA0000;
const COL_TEXT_WHITE: u32 = 0xFFFFFF;

/// Dictates the high-level routing in the `main.rs` loop.
pub enum AppState {
    StartScreen,
    Playing,
    GameOver,
    GamePaused,
    Controls,
    Quit,
}

/// Simple layout container for button definitions.
struct ButtonDef {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    text: String,
}

/// Holds the minimal state required for the IMGUI.
/// Since `minifb` natively handles keyboard debouncing via `KeyRepeat::No`,
/// we only need to manually track the mouse state for edge detection (clicks).
pub struct Menu {
    was_mouse_down: bool,
    waiting_for_key: Option<String>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            was_mouse_down: false,
            waiting_for_key: None,
        }
    }

    /// Renders the main menu and handles user interactions.
    pub fn update_and_draw_start_menu(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        renderer_data: &RendererData,
        audio: &mut Audio,
    ) -> AppState {
        // Clear the buffer from the previous frame
        buffer.fill(COL_BG_MAIN);

        // Fetch user inputs
        let (mx, my, clicked) = self.get_mouse_state(window);
        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);
        let click2 = window.is_key_pressed(Key::Key2, KeyRepeat::No);
        let click3 = window.is_key_pressed(Key::Key3, KeyRepeat::No);
        let click4 = window.is_key_pressed(Key::Key4, KeyRepeat::No);

        // Render titles
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "DOOMSQUAD",
            50,
            6,
            COL_TITLE_RED,
        );
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "MAIN MENU",
            105,
            2,
            COL_TEXT_WHITE,
        );

        // Define button layouts
        let btn_start = ButtonDef {
            x: 230,
            y: 130,
            w: 340,
            h: 45,
            text: "[1] NEW GAME".to_string(),
        };
        let btn_load = ButtonDef {
            x: 230,
            y: 185,
            w: 340,
            h: 45,
            text: "[2] LOAD MAP".to_string(),
        };

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 240,
            w: 340,
            h: 45,
            text: audio_text.to_string(),
        };

        
        let btn_controls = ButtonDef {
            x: 230,
            y: 295,
            w: 340,
            h: 45,
            text: "[4] CONTROLS".to_string(),
        };

        let exit_key_str = Self::key_to_string(game.bindings.menu_escape);
        let btn_quit = ButtonDef {
            x: 230,
            y: 350,
            w: 340,
            h: 45,
            text: format!("[{}] QUIT GAME", exit_key_str),
        };

        // Interaction processing
        if self.draw_flat_button(buffer, &btn_start, mx, my) && clicked || click1 {
            *game = Game::new_game(renderer_data).unwrap(); // ! this unwrap is intentionally accepted
            game.map_swap(renderer_data, 1); // Jump straight to map 1 (map 0 is reserved for testing)
            return AppState::Playing;
        }

        #[allow(clippy::collapsible_if)]
        if self.draw_flat_button(buffer, &btn_load, mx, my) && clicked || click2 {
            // Read saved map index from disk and safely parse it
            if let Ok(content) = fs::read_to_string("savegame.txt")
                && let Ok(index) = content.trim().parse::<usize>()
            {
                *game = Game::new_game(renderer_data).unwrap(); // ! this unwrap is intentionally accepted
                game.map_swap(renderer_data, index);
                return AppState::Playing;
            }
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_controls, mx, my)  && clicked || click4 {
            return AppState::Controls;
        }

        if self.draw_flat_button(buffer, &btn_quit, mx, my) && clicked {
            return AppState::Quit;
        }

        // Keep looping in the menu until an action changes the state
        AppState::StartScreen
    }

    /// Renders the death screen and allows the user to save their current level progress.
    pub fn update_and_draw_game_over(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
    ) -> AppState {
        buffer.fill(COL_BG_GAMEOVER);

        let (mx, my, clicked) = self.get_mouse_state(window);
        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "YOU DIED",
            80,
            6,
            COL_TITLE_RED,
        );

        let btn_save = ButtonDef {
            x: 230,
            y: 220,
            w: 340,
            h: 60,
            text: "[1] SAVE MAP INDEX".to_string(),
        };
        let btn_menu = ButtonDef {
            x: 230,
            y: 300,
            w: 340,
            h: 60,
            text: "[ESC] MAIN MENU".to_string(),
        };

        if self.draw_flat_button(buffer, &btn_save, mx, my) && clicked || click1 {
            // Persist the current level index to disk so it can be loaded later
            let _ = fs::write("savegame.txt", game.map_index.to_string());
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        AppState::GameOver
    }

    /// Renders the pause menu (triggered by pressing ESC during gameplay).
    pub fn update_and_draw_game_paused(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        audio: &mut Audio,
    ) -> AppState {
        buffer.fill(COL_BG_PAUSED);

        let (mx, my, clicked) = self.get_mouse_state(window);

        let click1 = window.is_key_pressed(Key::Key1, KeyRepeat::No);
        let click2 = window.is_key_pressed(Key::Key2, KeyRepeat::No);
        let click3 = window.is_key_pressed(Key::Key3, KeyRepeat::No);
        let click4 = window.is_key_pressed(Key::Key4, KeyRepeat::No);

        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "GAME PAUSED",
            40,
            6,
            COL_TITLE_BLUE,
        );

        let audio_text = if !audio.is_muted {
            "[3] SOUND: ON"
        } else {
            "[3] SOUND: OFF"
        };

        // Etwas kompakter gestaltet, damit alles perfekt ins Pausenmenü passt
        let btn_resume = ButtonDef {
            x: 230,
            y: 120,
            w: 340,
            h: 45,
            text: "[1] RESUME GAME".to_string(),
        };
        let btn_save = ButtonDef {
            x: 230,
            y: 175,
            w: 340,
            h: 45,
            text: "[2] SAVE & EXIT".to_string(),
        };
        let btn_audio = ButtonDef {
            x: 230,
            y: 230,
            w: 340,
            h: 45,
            text: audio_text.to_string(),
        };
        let btn_controls = ButtonDef {
            x: 230,
            y: 285,
            w: 340,
            h: 45,
            text: "[4] CONTROLS".to_string(),
        };

        let exit_key_str = Self::key_to_string(game.bindings.menu_escape);
        let btn_menu = ButtonDef {
            x: 230,
            y: 340,
            w: 340,
            h: 45,
            text: format!("[{}] MAIN MENU", exit_key_str),
        };

        if self.draw_flat_button(buffer, &btn_resume, mx, my) && clicked || click1 {
            return AppState::Playing;
        }

        if self.draw_flat_button(buffer, &btn_save, mx, my) && clicked || click2 {
            let _ = std::fs::write("savegame.txt", game.map_index.to_string());
            return AppState::StartScreen;
        }

        if self.draw_flat_button(buffer, &btn_audio, mx, my) && clicked || click3 {
            audio.set_muted(!audio.is_muted);
            return AppState::GamePaused;
        }

        if self.draw_flat_button(buffer, &btn_controls, mx, my) && clicked || click4 {
            return AppState::Controls;
        }

        if self.draw_flat_button(buffer, &btn_menu, mx, my) && clicked {
            return AppState::StartScreen;
        }

        AppState::GamePaused
    }

// Klartextnamen für Keybinds
    fn key_to_string(key_val: u32) -> String {
        let real_key = KeyBindings::to_key(key_val);
        let raw = format!("{:?}", real_key);
        raw.replace("Key::", "").replace("Key", "")
    }

    // Renders the controls menu screen in two columns with clean key names & interactive rebinding
    pub fn update_and_draw_controls(
        &mut self,
        window: &mut Window,
        buffer: &mut [u32],
        game: &mut Game,
        _audio: &mut Audio,
    ) -> AppState {
        buffer.fill(COL_BG_PAUSED);

        let (mx, my, clicked) = self.get_mouse_state(window);
        let escape_clicked = window.is_key_pressed(KeyBindings::to_key(game.bindings.menu_escape), KeyRepeat::No);

        // Waiting for a new key press if user clicked an option
        if let Some(ref key_name) = self.waiting_for_key.clone() {
            Self::draw_text_centered(buffer, SCREEN_WIDTH, SCREEN_HEIGHT, "PRESS A NEW KEY...", 180, 2, COL_TITLE_RED);
            Self::draw_text_centered(buffer, SCREEN_WIDTH, SCREEN_HEIGHT, &format!("For: {}", key_name), 220, 2, COL_TEXT_WHITE);

            let keys = window.get_keys_pressed(KeyRepeat::No);
            if let Some(&new_key) = keys.first() {
                let new_key_code = new_key as u32;

                match key_name.as_str() {
                    "FORWARD" => game.bindings.forward = new_key_code,
                    "BACKWARD" => game.bindings.backward = new_key_code,
                    "LEFT" => game.bindings.left = new_key_code,
                    "RIGHT" => game.bindings.right = new_key_code,
                    "SPRINT" => game.bindings.sprint = new_key_code,
                    "JUMP" => game.bindings.jump = new_key_code,
                    "CROUCH" => game.bindings.crouch = new_key_code,
                    "ROCKET JUMP" => game.bindings.rocket_jump = new_key_code,
                    "INTERACT" => game.bindings.interact = new_key_code,
                    "AIM UP" => game.bindings.aim_up = new_key_code,
                    "AIM DOWN" => game.bindings.aim_down = new_key_code,
                    "TURN LEFT" => game.bindings.turn_left = new_key_code,
                    "TURN RIGHT" => game.bindings.turn_right = new_key_code,
                    "TOGGLE AIM" => game.bindings.toggle_aim = new_key_code,
                    "FIRE BULLET" => game.bindings.fire_bullet = new_key_code,
                    "FIRE ARROW" => game.bindings.fire_arrow = new_key_code,
                    "GODMODE" => game.bindings.godmode = new_key_code,
                    "LOCK ENEMIES" => game.bindings.enemy_lock = new_key_code,
                    "EXIT" => game.bindings.menu_escape = new_key_code,
                    _ => {}
                }

                let _ = game.bindings.save();
                self.waiting_for_key = None;
            }
            return AppState::Controls;
        }

        // Title
        Self::draw_text_centered(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "CONTROLS (CLICK TO REBIND)",
            15,
            2,
            COL_TITLE_BLUE,
        );

        // List of Keybindings
        let bindings = &game.bindings;
        let controls_data = vec![
            ("FORWARD", bindings.forward),
            ("BACKWARD", bindings.backward),
            ("LEFT", bindings.left),
            ("RIGHT", bindings.right),
            ("SPRINT", bindings.sprint),
            ("JUMP", bindings.jump),
            ("CROUCH", bindings.crouch),
            ("ROCKET JUMP", bindings.rocket_jump),
            ("INTERACT", bindings.interact),
            ("AIM UP", bindings.aim_up),
            ("AIM DOWN", bindings.aim_down),
            ("TURN LEFT", bindings.turn_left),
            ("TURN RIGHT", bindings.turn_right),
            ("TOGGLE AIM", bindings.toggle_aim),
            ("FIRE BULLET", bindings.fire_bullet),
            ("FIRE ARROW", bindings.fire_arrow),
            ("GODMODE", bindings.godmode),
            ("LOCK ENEMIES", bindings.enemy_lock),
            ("EXIT", bindings.menu_escape),
        ];

        // Split into 2 columns
        let mid_point = (controls_data.len() + 1) / 2;
        let left_column = &controls_data[..mid_point];
        let right_column = &controls_data[mid_point..];

        // Helper to draw clickable columns
        let mut draw_clickable_column = |column: &[(&str, u32)], start_x: usize| {
            let mut y = 55;
            for (name, val) in column {
                let key_str = Self::key_to_string(*val);
                let display_text = format!("{}: {}", name, key_str);

                let btn = ButtonDef {
                    x: start_x,
                    y,
                    w: 340,
                    h: 22,
                    text: display_text,
                };

                if self.draw_flat_button(buffer, &btn, mx, my) && clicked {
                    self.waiting_for_key = Some(name.to_string());
                }

                y += 26;
            }
        };

        draw_clickable_column(left_column, 40);
        draw_clickable_column(right_column, 420);

        let escape_key_str = Self::key_to_string(game.bindings.menu_escape);

        let btn_back = ButtonDef {
            x: 230,
            y: 410,
            w: 340,
            h: 30,
            text: format!("[{}] BACK TO MENU", escape_key_str),
        };

        if self.draw_flat_button(buffer, &btn_back, mx, my) && clicked || escape_clicked {
            return AppState::StartScreen;
        }

        AppState::Controls
    }

    /// Fetches mouse coordinates and performs edge detection (debouncing) for left-clicks.
    fn get_mouse_state(&mut self, window: &Window) -> (usize, usize, bool) {
        // Clamp prevents out-of-bounds coordinates if the mouse leaves the window frame
        let mouse_pos = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let is_mouse_down = window.get_mouse_down(MouseButton::Left);

        let clicked = is_mouse_down && !self.was_mouse_down;
        self.was_mouse_down = is_mouse_down;

        (mouse_pos.0 as usize, mouse_pos.1 as usize, clicked)
    }

    /// Draws a rectangular button and its text onto the buffer.
    /// Returns `true` if the mouse cursor is currently hovering over it.
    fn draw_flat_button(&self, buffer: &mut [u32], btn: &ButtonDef, mx: usize, my: usize) -> bool {
        // Simple check for hover detection
        let hover = mx >= btn.x && mx <= (btn.x + btn.w) && my >= btn.y && my <= (btn.y + btn.h);
        let bg_color = if hover { COL_BTN_HOVER } else { COL_BTN_NORMAL };

        // Rasterize the rectangle into the 1D pixel buffer
        for row in btn.y..(btn.y + btn.h) {
            for col in btn.x..(btn.x + btn.w) {
                // Safety check to prevent panic from out-of-bounds memory writes
                if row < SCREEN_HEIGHT && col < SCREEN_WIDTH {
                    buffer[row * SCREEN_WIDTH + col] = bg_color;
                }
            }
        }

        // Center the text horizontally and vertically inside the button bounds
        let text_width = btn.text.len() * 8 * 2;
        let text_x = btn.x + btn.w.saturating_sub(text_width) / 2;
        let text_y = btn.y + btn.h.saturating_sub(16) / 2;

        Self::draw_text(
            buffer,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &btn.text,
            text_x,
            text_y,
            2,
            COL_TEXT_WHITE,
        );

        hover
    }

    /// Helper to align a given text string exactly in the horizontal center of the screen.
    fn draw_text_centered(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        y: usize,
        scale: usize,
        color: u32,
    ) {
        // Each character in the font is 8 pixels wide, multiplied by the scaling factor
        let text_width = text.len() * 8 * scale;
        let x = screen_width.saturating_sub(text_width) / 2;

        Self::draw_text(
            buffer,
            screen_width,
            screen_height,
            text,
            x,
            y,
            scale,
            color,
        );
    }

    /// Software text rasterizer using the `font8x8` library.
    /// It translates strings into raw pixel data and pushes them to the 1D buffer.
    #[allow(clippy::too_many_arguments)]
    fn draw_text(
        buffer: &mut [u32],
        screen_width: usize,
        screen_height: usize,
        text: &str,
        start_x: usize,
        start_y: usize,
        scale: usize,
        color: u32,
    ) {
        let mut cursor_x = start_x;

        for c in text.chars() {
            if let Some(glyph) = BASIC_FONTS.get(c) {
                #[allow(clippy::needless_range_loop)]
                for y in 0..8 {
                    for x in 0..8 {
                        // Bitwise check: if the specific bit in the glyph byte is a 1, a pixel needs to be drawn
                        if (glyph[y] & (1 << x)) != 0 {
                            // Nearest-neighbor scaling (draws larger 2D blocks for each active pixel based on the `scale` factor)
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    let px = cursor_x + x * scale + dx;
                                    let py = start_y + y * scale + dy;

                                    if px < screen_width && py < screen_height {
                                        // Flatten 2D coordinate into a 1D array index
                                        buffer[py * screen_width + px] = color;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Advance the cursor by the width of the character to prepare for the next letter
            cursor_x += 8 * scale;
        }
    }
}
