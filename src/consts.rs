const LEFT_MOUSE: u32 = 272;
const RIGHT_MOUSE: u32 = 273;
const MIDDLE_MOUSE: u32 = 274;

pub fn wayland2bar(button: u32) -> Option<u32> {
    Some(match button {
        LEFT_MOUSE => 1,
        MIDDLE_MOUSE => 3,
        RIGHT_MOUSE => 2,
        _ => return None,
    })
}
