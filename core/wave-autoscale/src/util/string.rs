pub fn make_ascii_titlecase(s: &String) -> String {
    match s.get(..1) {
        None => String::new(),
        Some(first_char) => first_char.to_uppercase() + s.get(1..).unwrap_or(""),
    }
}
