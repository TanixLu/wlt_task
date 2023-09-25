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

pub fn replace_password(
    text: impl AsRef<str>,
    password: impl AsRef<str>,
    to: impl AsRef<str>,
) -> String {
    if !password.as_ref().is_empty() {
        let encoded_password = urlencoding::encode(password.as_ref()).to_string();
        text.as_ref()
            .replace(password.as_ref(), to.as_ref())
            .replace(&encoded_password, to.as_ref())
    } else {
        text.as_ref().to_owned()
    }
}
