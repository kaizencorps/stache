
// checks that a given string contains only lowercase letters and numbers
pub fn is_valid_name(s: &str, require_lower: bool) -> bool {
    let len = s.len();
    if len > 32 || len < 2 {
        return false;
    }
    s.chars().all(|c| !c.is_whitespace()  && (c.is_ascii_lowercase() || c.is_ascii_digit() || (!require_lower && c.is_ascii_uppercase())))
}
