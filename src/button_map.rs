use evdev_rs::enums::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref BUTTON_MAP: HashMap<EV_KEY, EV_KEY> = {
        HashMap::from([
            // A, B, Y, X
            (EV_KEY::BTN_SOUTH, EV_KEY::KEY_A),
            (EV_KEY::BTN_EAST,  EV_KEY::KEY_B),
            (EV_KEY::BTN_WEST,  EV_KEY::KEY_Y),
            (EV_KEY::BTN_NORTH, EV_KEY::KEY_X),

            // Start, Select
            (EV_KEY::BTN_START,  EV_KEY::KEY_ENTER),
            (EV_KEY::BTN_SELECT, EV_KEY::KEY_BACKSPACE),

            // Left/right shoulder button
            (EV_KEY::BTN_TL, EV_KEY::KEY_L),
            (EV_KEY::BTN_TR, EV_KEY::KEY_R),
        ])
    };
}

pub const DPAD_MAP_LEFT: EV_KEY  = EV_KEY::KEY_LEFT;
pub const DPAD_MAP_RIGHT: EV_KEY = EV_KEY::KEY_RIGHT;
pub const DPAD_MAP_UP: EV_KEY    = EV_KEY::KEY_UP;
pub const DPAD_MAP_DOWN: EV_KEY  = EV_KEY::KEY_DOWN;

