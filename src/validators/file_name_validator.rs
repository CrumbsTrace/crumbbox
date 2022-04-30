pub fn validate_file_name(s: &str) -> Result<(), String> {
    let is_empty_or_whitespace = s.trim().is_empty();
    let is_too_long = s.len() > 255;
    let contains_control_characters = s.chars().any(|c| c.is_control());
    let invalid_names = vec!["..", "/", "\0"];

    if is_empty_or_whitespace
        || is_too_long
        || contains_control_characters
        || invalid_names.contains(&s)
    {
        Err(format!("Invalid file name: {}", s))
    } else {
        Ok(())
    }
}
