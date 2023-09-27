use aes_gcm_siv::{aead::Aead, Aes256GcmSiv, Nonce};
use blake2::{Blake2s256, Digest};

pub type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn get_str_between(
    text: &str,
    left: impl AsRef<str>,
    right: impl AsRef<str>,
) -> AnyResult<&str> {
    let left = left.as_ref();
    let right = right.as_ref();
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

pub fn get_machine_uid() -> AnyResult<String> {
    machine_uid::get().map_err(|e| e.to_string().into())
}

fn str_to_256bits(s: impl AsRef<str>) -> [u8; 32] {
    let mut hasher = Blake2s256::new();
    hasher.update(s.as_ref().as_bytes());
    hasher.finalize().into()
}

fn cipher_new(key: impl AsRef<[u8]>) -> AnyResult<Aes256GcmSiv> {
    use aes_gcm_siv::aead::KeyInit;
    Aes256GcmSiv::new_from_slice(key.as_ref()).map_err(|e| e.into())
}

fn default_nonce() -> Nonce {
    Nonce::from(b"\0\0\0\0\0\0\0\0\0\0\0\0".to_owned())
}

/// "plain_text" --"key"--> "646839e2fc6b3c89019e92599c8da37475b295045ced51996f62"
pub fn str_encode(plaintext: impl AsRef<str>, key: impl AsRef<str>) -> AnyResult<String> {
    cipher_new(str_to_256bits(key))?
        .encrypt(&default_nonce(), plaintext.as_ref().as_bytes())
        .map(hex::encode)
        .map_err(|e| e.to_string().into())
}

/// "646839e2fc6b3c89019e92599c8da37475b295045ced51996f62" --"key"--> "plain_text"
pub fn str_decode(ciphertext: impl AsRef<str>, key: impl AsRef<str>) -> AnyResult<String> {
    cipher_new(str_to_256bits(key))?
        .decrypt(
            &default_nonce(),
            hex::decode(ciphertext.as_ref())?.as_slice(),
        )
        .map(|b| String::from_utf8_lossy(b.as_slice()).to_string())
        .map_err(|e| e.to_string().into())
}
