pub fn split_key_value(line: &str) -> Option<(&str, &str)> {
    line.find('=').map(|pos| {
        let (lh, rh) = line.split_at(pos);
        (lh.trim(), rh[1..].trim())
    })
}
