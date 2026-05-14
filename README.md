<h1 align="center">Kaho</h1>

<p align="center"><img src="logo.png" alt="VoidChan Logo" width="200"/></p>

<p align="center">A <b>Rust-based</b> library for interacting with Stoat.</p>

<p align="center">
<a href="https://kaho.2rkf.fun/docs/getting-started" target="_blank">Getting Started</a> · <a href="https://kaho.2rkf.fun/docs/examples" target="_blank">Examples</a> · <a href="https://stt.gg/g65YG8CA" target="_blank">Stoat</a>
</p>

> [!WARNING]
> This library is heavily under development. Bugs are expected.

## Installation

Kaho supports a MSRV of **Rust 1.76 or later**.

```toml
# Add crate to your Cargo.toml
[dependencies]
kaho = "*"
tokio = { version = "*", features = ["macros", "rt-multi-thread"] }
```

## Ping Pong Example

```rs
use kaho::{
    client::KahoClientBuilder,
    models::{GatewayEvent, MessageSend},
    KahoResult,
};

#[tokio::main]
async fn main() -> KahoResult<()> {
    let mut client = KahoClientBuilder::new()
        .token("TOKEN")
        .build()?;

    client.connect().await?;

    let mut events = client.events();

    while let Some(Ok(event)) = events.next().await {
        match event {
            GatewayEvent::Message(m) if m.content == "!ping" => {
                m.reply(
                    &client.http,
                    MessageSend {
                        content: "Pong!".into(),
                        ..Default::default()
                    },
                    false,
                )
                .await?;
            }
            _ => {}
        }
    }
    Ok(())
}

```

## License

Please refer to the [LICENSE](https://github.com/reinacchi/kaho/blob/master/LICENSE) file.
