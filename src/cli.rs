use {
    crate::{GmailClient, GmailClientConfig},
    anyhow::Context,
    clap::Parser,
    handlebars::Handlebars,
    serde_json::json,
    std::io::ErrorKind,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The email address to subscribe to
    #[arg(env = "ACCOUNT")]
    account: String,

    /// Enable this flag to turn on the auth flow.  If this flag isn't enabled
    /// and there are no cached credentials, then we will print the message in
    /// `auth_format` or a default message and then exit immediately.
    #[arg(short, long, env = "FORMAT")]
    auth: bool,

    /// The format string to use to display the results.  Use the `{total}`
    /// placeholder for the number of total messages, and use the `{unread}`
    /// placeholder for the number of unread messages.
    #[arg(short, long, env = "FORMAT")]
    format: Option<String>,

    /// The format string to use to display auth prompt.  Use the `{url}`
    /// placeholder for the auth URL to open in the browser to complete the auth
    /// flow.
    #[arg(long, env = "FORMAT")]
    auth_format: Option<String>,

    /// The gmail label to subscribe to
    #[arg(short, long, env = "LABEL", default_value = "INBOX")]
    label: String,

    /// The path to the file where auth tokens are stored.  Defaults to
    /// $XDG_STATE_HOME/gmail-new-mail-counter/<ACCOUNT>.token.json
    #[arg(long, env = "TOKEN_FILE")]
    token_file: Option<String>,

    /// The Google OAuth2 client ID
    #[arg(long, env = "CLIENT_ID")]
    client_id: String,

    /// The Google OAuth2 client secret
    #[arg(long, env = "CLIENT_SECRET")]
    client_secret: String,

    /// The Google OAuth2 project ID
    #[arg(long, env = "PROJECT_ID")]
    project_id: String,
}

#[tokio::main]
pub async fn cli() -> anyhow::Result<()> {
    load_dotenv().context("Invalid .env file")?;
    let Cli {
        account,
        auth,
        auth_format,
        client_id,
        client_secret,
        format,
        label,
        project_id,
        token_file,
    } = Cli::parse();

    let client = GmailClient::create(GmailClientConfig {
        account,
        auth_enabled: auth,
        auth_format,
        client_id,
        client_secret,
        project_id,
        token_file,
    })
    .await
    .context("Failed to create gmail client")?;

    let (total, unread) = client
        .get_messages_count(&label)
        .await
        .context("Failed to retrieve message count")?;

    match format {
        Some(format) => println!(
            "{}",
            Handlebars::new()
                .render_template(&format, &json!({"total": total, "unread": unread}))
                .context("Failed to format output")?
        ),
        None => println!("total: {total}, unread: {unread}"),
    }

    Ok(())
}

fn load_dotenv() -> dotenvy::Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => Ok(()),
        Err(dotenvy::Error::Io(err)) if (err.kind() == ErrorKind::NotFound) => Ok(()),
        Err(e) => Err(e),
    }
}
