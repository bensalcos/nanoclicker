use global_hotkey::hotkey::HotKey;
use nanoclick_core::BackendError;

// https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub(crate) fn virtual_key(hotkey: HotKey) -> Result<u32, BackendError> {
    let name = format!("{:?}", hotkey.key);
    if let Some(letter) = name.strip_prefix("Key")
        && letter.len() == 1
    {
        return Ok(u32::from(letter.as_bytes()[0]));
    }
    if let Some(digit) = name.strip_prefix("Digit")
        && digit.len() == 1
    {
        return Ok(u32::from(digit.as_bytes()[0]));
    }
    if let Some(number) = name.strip_prefix('F').and_then(|n| n.parse::<u32>().ok())
        && (1..=24).contains(&number)
    {
        return Ok(0x6f + number);
    }
    match name.as_str() {
        "Backspace" => Ok(0x08),
        "Tab" => Ok(0x09),
        "Enter" => Ok(0x0d),
        "Pause" => Ok(0x13),
        "CapsLock" => Ok(0x14),
        "Escape" => Ok(0x1b),
        "Space" => Ok(0x20),
        "PageUp" => Ok(0x21),
        "PageDown" => Ok(0x22),
        "End" => Ok(0x23),
        "Home" => Ok(0x24),
        "ArrowLeft" => Ok(0x25),
        "ArrowUp" => Ok(0x26),
        "ArrowRight" => Ok(0x27),
        "ArrowDown" => Ok(0x28),
        "PrintScreen" => Ok(0x2c),
        "Insert" => Ok(0x2d),
        "Delete" => Ok(0x2e),
        _ => Err(BackendError::new(
            "Use a letter, digit, F1–F24, Space, Enter, Tab, Escape or navigation key.",
        )),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maps_function_letters_and_navigation_keys() -> Result<(), Box<dyn std::error::Error>> {
        for (text, code) in [
            ("F1", 0x70),
            ("F6", 0x75),
            ("F24", 0x87),
            ("Ctrl+Shift+A", 0x41),
            ("Alt+9", 0x39),
            ("Home", 0x24),
        ] {
            assert_eq!(virtual_key(text.parse()?)?, code);
        }
        Ok(())
    }
    #[test]
    fn rejects_unimplemented_keys_instead_of_registering_the_wrong_key()
    -> Result<(), Box<dyn std::error::Error>> {
        assert!(virtual_key("AudioVolumeUp".parse()?).is_err());
        Ok(())
    }
}
