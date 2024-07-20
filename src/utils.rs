use std::fmt::Display;

use aes_gcm_siv::{aead::Aead, Aes256GcmSiv, Nonce};
use anyhow::Context;
use blake2::{Blake2s256, Digest};

pub fn get_str_between(
    text: &str,
    left: impl AsRef<str>,
    right: impl AsRef<str>,
) -> anyhow::Result<&str> {
    let left = left.as_ref();
    let right = right.as_ref();
    let left = text
        .find(left)
        .context(format!("left不在text中\nleft: {}\ntext: {}", left, text))?
        + left.len();
    let no_left = &text[left..];
    let right = no_left
        .find(right)
        .context(format!("right不在text中\nright: {}\ntext: {}", right, text))?;
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

fn get_machine_uid() -> anyhow::Result<String> {
    machine_uid::get().map_err(|e| anyhow::Error::msg(e.to_string()))
}

fn str_to_256bits(s: impl AsRef<str>) -> [u8; 32] {
    let mut hasher = Blake2s256::new();
    hasher.update(s.as_ref().as_bytes());
    hasher.finalize().into()
}

fn cipher_new(key: impl AsRef<[u8]>) -> anyhow::Result<Aes256GcmSiv> {
    use aes_gcm_siv::aead::KeyInit;
    Aes256GcmSiv::new_from_slice(key.as_ref()).map_err(|e| e.into())
}

fn default_nonce() -> Nonce {
    Nonce::from(b"\0\0\0\0\0\0\0\0\0\0\0\0".to_owned())
}

/// "plain_text" --"key"--> "646839e2fc6b3c89019e92599c8da37475b295045ced51996f62"
fn str_encrypt(plaintext: impl AsRef<str>) -> anyhow::Result<String> {
    cipher_new(str_to_256bits(get_machine_uid()?))?
        .encrypt(&default_nonce(), plaintext.as_ref().as_bytes())
        .map(hex::encode)
        .map_err(|e| anyhow::Error::msg(e.to_string()))
}

/// "646839e2fc6b3c89019e92599c8da37475b295045ced51996f62" --"key"--> "plain_text"
pub fn str_decrypt(ciphertext: impl AsRef<str>) -> anyhow::Result<String> {
    cipher_new(str_to_256bits(get_machine_uid()?))?
        .decrypt(
            &default_nonce(),
            hex::decode(ciphertext.as_ref())?.as_slice(),
        )
        .map(|b| String::from_utf8_lossy(b.as_slice()).to_string())
        .map_err(|e| anyhow::Error::msg(e.to_string()))
}

pub fn substr_encrypt(text: impl AsRef<str>, substr: impl AsRef<str>) -> anyhow::Result<String> {
    if !substr.as_ref().is_empty() && str_decrypt(substr.as_ref()).is_err() {
        Ok(text
            .as_ref()
            .replace(substr.as_ref(), &str_encrypt(substr.as_ref())?))
    } else {
        Ok(text.as_ref().to_owned())
    }
}

pub fn print_list(texts: impl IntoIterator<Item = impl Display>, start_index: i32) {
    let mut i = start_index;
    for s in texts {
        println!("{i}. {s}");
        i += 1;
    }
}

pub fn get_range_u32(l: u32, r: u32) -> u32 {
    loop {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("read line failed");
        match buf.trim().parse::<u32>() {
            Ok(i) if i >= l && i <= r => return i,
            _ => (),
        }
    }
}

pub fn input_key_to_continue(key: &str, text: &str) {
    println!("{text}");
    loop {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("failed to read line");
        if buf.trim() == key {
            break;
        }
    }
}

pub fn get_ipv6() -> anyhow::Result<String> {
    use reqwest::blocking::get;
    Ok(get("http://api6.ipify.org/")?.text()?)
}
