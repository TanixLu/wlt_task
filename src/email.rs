use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::log::log;
use crate::utils::AnyResult;

pub fn send_email(
    server: &str,
    username: &str,
    password: &str,
    email_to_list: &[String],
    subject: &str,
    body: &str,
) {
    let try_send_email = || -> AnyResult<()> {
        let creds = Credentials::new(username.to_owned(), password.to_owned());

        let mailer = SmtpTransport::relay(server)?.credentials(creds).build();

        let mut email = Message::builder().from(username.parse()?);
        if email_to_list.is_empty() {
            email = email.to(username.parse()?);
        } else {
            for mailbox_string in email_to_list.iter() {
                email = email.to(mailbox_string.parse()?);
            }
        }
        let email = email
            .subject(subject.to_owned())
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_owned())?;

        mailer.send(&email)?;

        Ok(())
    };

    if let Err(e) = try_send_email() {
        log(format!("发送邮件失败: {}", e));
    }
}
