// Raider
//
// Affiliates dashboard
// Copyright: 2018, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use lettre::smtp::authentication::Credentials;
use lettre::smtp::client::net::ClientTlsParameters;
use lettre::smtp::{ClientSecurity, ConnectionReuseParameters};
use lettre::{SmtpClient, SmtpTransport, Transport};
use lettre_email::Email;
use log;
use native_tls::TlsConnector;
use std::time::Duration;

use crate::APP_CONF;

pub struct EmailNotifier;

impl EmailNotifier {
    pub fn dispatch(to: &str, subject: String, body: &str) -> Result<(), bool> {
        // Build up the message text
        let mut message = String::new();

        message.push_str(body);
        message.push_str("\n\n--\n\n");

        message.push_str(&format!(
            "You receive this email because an event occured on your {} account at: {}",
            APP_CONF.branding.page_title,
            APP_CONF.branding.page_url.as_str()
        ));

        log::debug!("will send email notification with message: {}", &message);

        // Build up the email
        let email_message = Email::builder()
            .to(to)
            .from((
                APP_CONF.email.from.as_str(),
                APP_CONF.branding.page_title.as_str(),
            ))
            .subject(subject)
            .text(message)
            .build()
            .or(Err(true))?;

        // Deliver the message
        return acquire_transport(
            &APP_CONF.email.smtp_host,
            APP_CONF.email.smtp_port,
            APP_CONF.email.smtp_username.to_owned(),
            APP_CONF.email.smtp_password.to_owned(),
            APP_CONF.email.smtp_encrypt,
        )
        .map(|mut transport| transport.send(email_message.into()))
        .and(Ok(()))
        .or(Err(true));
    }
}

fn acquire_transport(
    smtp_host: &str,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    smtp_encrypt: bool,
) -> Result<SmtpTransport, ()> {
    let mut security = ClientSecurity::None;

    if smtp_encrypt == true {
        if let Ok(connector) = TlsConnector::new() {
            security = ClientSecurity::Required(ClientTlsParameters {
                connector: connector,
                domain: smtp_host.to_string(),
            });
        }

        // Do not deliver email if TLS context cannot be acquired (prevents unencrypted emails \
        //   to be sent)
        if let ClientSecurity::None = security {
            log::error!("could not build smtp encrypted connector");

            return Err(());
        }
    }

    match SmtpClient::new((smtp_host, smtp_port), security) {
        Ok(client) => {
            let mut client = client
                .timeout(Some(Duration::from_secs(5)))
                .connection_reuse(ConnectionReuseParameters::NoReuse);

            match (smtp_username, smtp_password) {
                (Some(smtp_username_value), Some(smtp_password_value)) => {
                    client = client
                        .credentials(Credentials::new(smtp_username_value, smtp_password_value));
                }
                _ => {}
            }

            Ok(client.transport())
        }
        Err(err) => {
            log::error!("could not acquire smtp transport: {}", err);

            Err(())
        }
    }
}
