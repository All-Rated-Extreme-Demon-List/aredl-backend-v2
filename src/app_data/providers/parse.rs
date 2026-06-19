pub fn is_ascii_alphanumeric(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

pub fn is_ascii_digits(value: &str, min_len: usize, max_len: usize) -> bool {
    (min_len..=max_len).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_digit())
}

pub fn is_ascii_id(value: &str, min_len: usize, max_len: usize) -> bool {
    (min_len..=max_len).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

pub fn is_twitch_timestamp(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'h' | b'm' | b's'))
}

pub fn is_youtube_timestamp(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    if is_ascii_digits(value, 1, 10) {
        return true;
    }

    let mut rest = value;
    let mut matched_component = false;

    if let Some(next) = consume_numbered_unit(rest, b'h') {
        rest = next;
        matched_component = true;
    }

    if let Some(next) = consume_numbered_unit(rest, b'm') {
        rest = next;
        matched_component = true;
    }

    if let Some(next) = consume_numbered_unit(rest, b's') {
        rest = next;
        matched_component = true;
    }

    matched_component && rest.is_empty()
}

fn consume_numbered_unit(value: &str, unit: u8) -> Option<&str> {
    let bytes = value.as_bytes();
    let digit_count = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();

    if !(1..=4).contains(&digit_count) || bytes.get(digit_count) != Some(&unit) {
        return None;
    }

    value.get(digit_count + 1..)
}
