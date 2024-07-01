use {
    google_gmail1::{
        api::Label,
        hyper::{body::Body, client::connect::HttpConnector, Client},
        hyper_rustls::{HttpsConnector, HttpsConnectorBuilder},
        oauth2::{
            authenticator::Authenticator, authenticator_delegate::InstalledFlowDelegate,
            ApplicationSecret, InstalledFlowAuthenticator, InstalledFlowReturnMethod,
        },
        Gmail,
    },
    handlebars::Handlebars,
    serde_json::json,
    std::{
        future::Future,
        io, mem,
        path::{Path, PathBuf},
        pin::Pin,
    },
    xdg::{BaseDirectories, BaseDirectoriesError},
};

const XDG_BASE_DIR_NAME: &str = "gmail-new-mail-counter";
const TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
const AUTH_URI: &str = "https://accounts.google.com/o/oauth2/auth";
const REDIRECT_URI: &str = "http://localhost";
const AUTH_PROVIDER_X509_CERT_URL: &str = "https://www.googleapis.com/oauth2/v1/certs";

pub struct GmailClient {
    client: Gmail<HttpsConnector<HttpConnector>>,
}

pub struct GmailClientConfig {
    pub account: String,
    pub auth_format: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub project_id: String,
    pub token_file: Option<String>,
}

impl GmailClient {
    pub async fn create(mut config: GmailClientConfig) -> Result<Self, CreateGmailClientError> {
        Ok(Self {
            client: Gmail::new(build_client()?, build_auth(&mut config).await?),
        })
    }

    pub async fn get_messages_count(
        &self,
        label: &str,
    ) -> Result<(i32, i32), GetMessageCountError> {
        let Label {
            messages_total,
            messages_unread,
            ..
        } = self.client.users().labels_get("me", label).doit().await?.1;

        Ok((
            messages_total.ok_or(GetMessageCountError::MissingTotal)?,
            messages_unread.ok_or(GetMessageCountError::MissingUnread)?,
        ))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateGmailClientError {
    #[error("Failed to find system SSL certificates")]
    NoFindSSLCerts { source: io::Error },

    #[error("Failed to create authenticator")]
    CantCreateAuthenticator { source: io::Error },

    #[error("Failed to look up XDG user dirs")]
    CantGetXDGDirectories {
        #[from]
        source: BaseDirectoriesError,
    },

    #[error("Failed to create state directory")]
    CantCreateStateDir { source: io::Error },
}

fn build_client() -> Result<Client<HttpsConnector<HttpConnector>, Body>, CreateGmailClientError> {
    Ok(Client::builder().build(
        HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|source| CreateGmailClientError::NoFindSSLCerts { source })?
            .https_or_http()
            .enable_http1()
            .build(),
    ))
}

async fn build_auth(
    config: &mut GmailClientConfig,
) -> Result<Authenticator<HttpsConnector<HttpConnector>>, CreateGmailClientError> {
    let mut builder = InstalledFlowAuthenticator::builder(
        build_secret(config),
        InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk(get_token_file(config)?)
    .login_hint(mem::take(&mut config.account));

    if let Some(auth_format) = mem::take(&mut config.auth_format) {
        builder = builder.flow_delegate(Box::new(FlowDelegate(auth_format)));
    }

    builder
        .build()
        .await
        .map_err(|source| CreateGmailClientError::CantCreateStateDir { source })
}

fn build_secret(config: &mut GmailClientConfig) -> ApplicationSecret {
    ApplicationSecret {
        client_id: mem::take(&mut config.client_id),
        client_secret: mem::take(&mut config.client_secret),
        token_uri: TOKEN_URI.into(),
        auth_uri: AUTH_URI.into(),
        redirect_uris: vec![REDIRECT_URI.into()],
        project_id: Some(mem::take(&mut config.project_id)),
        client_email: None,
        auth_provider_x509_cert_url: Some(AUTH_PROVIDER_X509_CERT_URL.into()),
        client_x509_cert_url: None,
    }
}

fn get_token_file(config: &mut GmailClientConfig) -> Result<PathBuf, CreateGmailClientError> {
    match mem::take(&mut config.token_file) {
        Some(file) => Ok(file.into()),
        None => Ok(BaseDirectories::new()?
            .place_state_file(
                Path::new(XDG_BASE_DIR_NAME).join(format!("{}.token.json", config.account)),
            )
            .map_err(|source| CreateGmailClientError::CantCreateStateDir { source })?),
    }
}

struct FlowDelegate(String);

impl InstalledFlowDelegate for FlowDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(present_user_url(&self.0, url, need_code))
    }
}

async fn present_user_url<'a>(
    format: &'a str,
    url: &'a str,
    need_code: bool,
) -> Result<String, String> {
    if need_code {
        Err("A code was required but we don't handle codes here!".into())
    } else {
        println!(
            "{}",
            Handlebars::new()
                .render_template(format, &json!({ "url": String::from(url) }))
                .map_err(|_| "Failed to format output")?
        );
        Ok(String::new())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetMessageCountError {
    #[error("Failed to fetch message counts")]
    FailedToFetch {
        #[from]
        source: google_gmail1::Error,
    },

    #[error("Google response is missing a total message count")]
    MissingTotal,

    #[error("Google response is missing an unread message count")]
    MissingUnread,
}
