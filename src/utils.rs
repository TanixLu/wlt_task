pub type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn find_str_between<'a>(text: &'a str, left: &str, right: &str) -> AnyResult<&'a str> {
    let left_loc = text
        .find(left)
        .ok_or(format!("在{}中没有找到{}", text, left))?;
    let no_left = &text[left_loc + left.len()..];
    let right_loc = no_left
        .find(right)
        .ok_or(format!("在{}中没有找到{}", text, right))?;
    Ok(&no_left[..right_loc])
}
