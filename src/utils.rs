pub type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn get_str_between<'a>(text: &'a str, left: &str, right: &str) -> AnyResult<&'a str> {
    let left = text
        .find(left)
        .ok_or(format!("left不在text中\nleft: {}\ntext: {}", left, text))?
        + left.len();
    let no_left = &text[left..];
    let right = no_left
        .find(right)
        .ok_or(format!("right不在text中\nright: {}\ntext: {}", right, text))?;
    Ok(&no_left[..right])
}
