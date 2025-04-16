pub mod listener;

use redev::Button;

#[derive(Debug, thiserror::Error)]
#[error("Unrecognized key")]
pub struct UnknownButton;

pub fn mouse_button_code_convert(rdev_button: Button) -> Result<u32, UnknownButton> {
    match rdev_button {
        Button::Left => Ok(0),
        Button::Middle => Ok(2),
        Button::Right => Ok(1),
        Button::Unknown(code) => match code {
            1 | 8 | 19 => Ok(3),
            2 | 9 | 20 => Ok(4),
            6 => Ok(6),
            7 => Ok(7),
            _ => Err(UnknownButton),
        },
    }
}
