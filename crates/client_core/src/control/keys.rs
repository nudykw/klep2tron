//! Key-name parsing for `POST /key`.

pub(super) fn key_from_name(name: &str) -> Option<super::KeyCode> {
    use super::KeyCode::*;
    Some(match name {
        "ArrowUp" | "Up" => ArrowUp,
        "ArrowDown" | "Down" => ArrowDown,
        "ArrowLeft" | "Left" => ArrowLeft,
        "ArrowRight" | "Right" => ArrowRight,
        "Enter" | "Return" => Enter,
        "Space" => Space,
        "Escape" | "Esc" => Escape,
        "Tab" => Tab,
        "Backspace" => Backspace,
        "Delete" => Delete,
        "ShiftLeft" | "Shift" => ShiftLeft,
        "ControlLeft" | "Ctrl" => ControlLeft,
        "AltLeft" | "Alt" => AltLeft,
        "F1" => F1,
        "F2" => F2,
        "F3" => F3,
        "F4" => F4,
        "F5" => F5,
        "KeyA" => KeyA,
        "KeyB" => KeyB,
        "KeyC" => KeyC,
        "KeyD" => KeyD,
        "KeyE" => KeyE,
        "KeyF" => KeyF,
        "KeyG" => KeyG,
        "KeyH" => KeyH,
        "KeyI" => KeyI,
        "KeyJ" => KeyJ,
        "KeyK" => KeyK,
        "KeyL" => KeyL,
        "KeyM" => KeyM,
        "KeyN" => KeyN,
        "KeyO" => KeyO,
        "KeyP" => KeyP,
        "KeyQ" => KeyQ,
        "KeyR" => KeyR,
        "KeyS" => KeyS,
        "KeyT" => KeyT,
        "KeyU" => KeyU,
        "KeyV" => KeyV,
        "KeyW" => KeyW,
        "KeyX" => KeyX,
        "KeyY" => KeyY,
        "KeyZ" => KeyZ,
        "Digit0" => Digit0,
        "Digit1" => Digit1,
        "Digit2" => Digit2,
        "Digit3" => Digit3,
        "Digit4" => Digit4,
        "Digit5" => Digit5,
        "Digit6" => Digit6,
        "Digit7" => Digit7,
        "Digit8" => Digit8,
        "Digit9" => Digit9,
        _ => {
            if name.len() == 1 {
                let c = name.chars().next().unwrap();
                if c.is_ascii_alphabetic() {
                    return key_from_char(c);
                }
                if c.is_ascii_digit() {
                    return key_from_digit(c);
                }
            }
            return None;
        }
    })
}

fn key_from_char(c: char) -> Option<super::KeyCode> {
    use super::KeyCode::*;
    Some(match c.to_ascii_uppercase() {
        'A' => KeyA, 'B' => KeyB, 'C' => KeyC, 'D' => KeyD, 'E' => KeyE, 'F' => KeyF,
        'G' => KeyG, 'H' => KeyH, 'I' => KeyI, 'J' => KeyJ, 'K' => KeyK, 'L' => KeyL,
        'M' => KeyM, 'N' => KeyN, 'O' => KeyO, 'P' => KeyP, 'Q' => KeyQ, 'R' => KeyR,
        'S' => KeyS, 'T' => KeyT, 'U' => KeyU, 'V' => KeyV, 'W' => KeyW, 'X' => KeyX,
        'Y' => KeyY, 'Z' => KeyZ,
        _ => return None,
    })
}

fn key_from_digit(c: char) -> Option<super::KeyCode> {
    use super::KeyCode::*;
    Some(match c {
        '0' => Digit0, '1' => Digit1, '2' => Digit2, '3' => Digit3, '4' => Digit4,
        '5' => Digit5, '6' => Digit6, '7' => Digit7, '8' => Digit8, '9' => Digit9,
        _ => return None,
    })
}
