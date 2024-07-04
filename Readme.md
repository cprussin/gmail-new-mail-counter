# Gmail New Mail Counter

This is a very simple program that simply counts mail out of your gmail mailbox
and returns the counts, along with the unread counts.  Optionally, you can
supply a custom formatter to format the results.

The intended use for this is to drive an indicator on something like
[waybar](https://github.com/Alexays/Waybar) or similar.

## Obtaining

If you use nix, you can use the [included flake](./flake.nix), e.g. `nix run
github:cprussin/gmail-new-mail-counter`.

If you don't use nix, you can build with
[cargo](https://doc.rust-lang.org/cargo/).  Just obtain Cargo however makes the
most sense on your platform, and then run `cargo build`.

## Setup

You will need to first set up an OAuth app in Google's developer console.  You
can do so by following [the guide on Google's support
site](https://support.google.com/cloud/answer/6158849?hl=en).

- When creating the credentials, select "Desktop app" as the "Application type".
- When setting up the consent screen, make sure you add the scope
  `https://www.googleapis.com/auth/gmail.metadata`

You will need the client ID, client secret, and project ID from the generated
credentials.  You can find those in the json payload you will download when you
create the credentials.

Next, you'll need to [enable the Gmail
api](https://console.cloud.google.com/flows/enableapi?apiid=gmail.googleapis.com).

## Running

You can either pass the client ID, client secret, project ID, and account you
want to count mail for as environment variables or you can pass them as CLI
arguments:

```
CLIENT_SECRET=foo CLIENT_ID=bar PROJECT_ID=baz ACCOUNT=someone@gmail.com gmail_new_mail_counter 
```

```
gmail_new_mail_counter --client-id bar --client-secret foo --project-id baz someone@gmail.com
```

Note you can mix & match CLI arguments and env variables if you need.

There are some additional arguments that are useful, including `--format` and
`--auth-format` to format the output messages, and `--label` to select which
label to count mail for.  See the `--help` for details.

## Example Waybar config

Add a custom module to your waybar config:

```
  "custom/email-counter": {
    "exec": "/path/to/get-email.sh",
    "on-click": "/path/to/browser https://mail.google.com",
    "restart-interval": 20,
    "return-type": "json"
  },
```

Place the following into `get-email.sh`:

```shell
#!/bin/sh
set -a
. /path/to/env/file/with/secrets
set +a
gmail_new_mail_counter \
  --format '{{#if (gt total 0)}}{"text":"✉ {{ unread }} / {{ total }}"{{#if (gt unread 0) }},"class":"unread"{{/if}}}{{/if}}' \
  --auth-format '{"text":"✉ "}' \
  <your email address>@gmail.com
```

Place the `CLIENT_SECRET`, `CLIENT_ID`, and `PROJECT_ID` env variables with the
values from your OAuth app credentials in `/path/to/env/file/with/secrets`, and
then run `gmail_new_mail_counter --auth <your email address>@gmail.com` to
initialize credentials.
