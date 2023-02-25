# gitevents

Gitevents is an abstraction on a git repository which allows you to receive
events when a repository has an update. Events can either be sent via. NATS, a
webhook via. an SDK.

## Installation

```bash
cargo init --bin mylistener

cargo add gitevents eyre
cargo add tokio --features full
```

## Usage

```rust
#[tokio::full]
async fn main() -> eyre::Result<()> {
  gitevents_sdk::listen("github.com/kjuulh/gitevents")
    .nats("nats://address")
    .webhook("http://localhost:3000/webhook")
    .action(async |event| -> eyre::Result<EventResp> {
        Ok(EventResp::Ack)
    })
    .execute()
    .await?;
}
```

It is possible to build extra handler using a normal trait extension method.
Follow the docs on how to do that.

## Hosting

The sdk will reconciliate by default once every 5 minutes. However, it needs a
place to run, this can either be locally, self-hosted or via. Our hosting
platform.

See [gitevents deploy](https://<insert-domain-here>/signup?ref=github). Using
our sdk you will get a faster reconciliation loop, and other useful features
such as logging, notifications and access to additional sdk plugins.
