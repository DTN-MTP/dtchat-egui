pub fn safe_id_display(id: &str) -> &str {
    if id.len() >= 8 {
        &id[..8]
    } else {
        id
    }
}
