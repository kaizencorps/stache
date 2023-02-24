
// checks that a given string contains only lowercase letters and numbers
pub fn is_valid_name(s: &str) -> bool {
    let len = s.len();
    if len > 32 || len == 0 {
        return false;
    }
    s.chars().all(|c| !c.is_whitespace()  && (c.is_ascii_lowercase() || c.is_ascii_digit()))
}
