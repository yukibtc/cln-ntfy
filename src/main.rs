// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use cln_plugin::options::{ConfigOption, Value};
use cln_plugin::{Builder, Error, Plugin};
use ntfy::{Dispatcher, Payload};
use tokio::sync::mpsc::{self, Sender};

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
struct PluginState {
    sender: Sender<Payload>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (sender, mut receiver) = mpsc::channel(1024);

    let state = PluginState { sender };

    let plugin = match Builder::new(state, tokio::io::stdin(), tokio::io::stdout())
        .option(ConfigOption::new(
            "clntfy-url",
            Value::String(String::new()),
            "ntfy url",
        ))
        .option(ConfigOption::new(
            "clntfy-topic",
            Value::String(String::from("cln-alerts")),
            "ntfy topic",
        ))
        .option(ConfigOption::new(
            "clntfy-username",
            Value::String(String::new()),
            "ntfy username",
        ))
        .option(ConfigOption::new(
            "clntfy-password",
            Value::String(String::new()),
            "ntfy password",
        ))
        .option(ConfigOption::new(
            "clntfy-proxy",
            Value::String(String::new()),
            "ntfy proxy",
        ))
        .subscribe("invoice_payment", invoice_payment_handler)
        .configure()
        .await?
    {
        Some(p) => p,
        None => return Ok(()),
    };

    let url: String = match plugin.option("clntfy-url") {
        Some(Value::String(url)) => {
            if url.is_empty() {
                panic!("'clntfy-url' required")
            } else {
                url
            }
        }
        _ => panic!("'clntfy-url' required"),
    };

    match plugin.option("clntfy-topic") {
        Some(Value::String(topic)) => {
            if topic.is_empty() {
                panic!("'clntfy-topic' required")
            }
        }
        _ => panic!("'clntfy-topic' required"),
    };

    let dispatcher = Dispatcher::builder(url)
        /* .credentials(Auth::new("username", "password")) // Add optional credentials
        .proxy("socks5h://127.0.0.1:9050") // Add optional proxy */
        .build()?; // Build dispatcher

    let plugin = plugin.start().await?;

    while let Some(payload) = receiver.recv().await {
        dispatcher.send(&payload).await?;
    }

    plugin.join().await?;

    Ok(())
}

async fn invoice_payment_handler(
    p: Plugin<PluginState>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::info!("Got a invoice payment notification: {v}");

    if let Some(Value::String(topic)) = p.option("clntfy-topic") {
        let amount_msat = &v["invoice_payment"]["amount_msat"];
        let amount: usize = serde_json::from_value(amount_msat.clone())?;
        let amount: u64 = (amount / 1000) as u64;

        let payload = Payload::new(topic)
            .message(format!("+{amount} sat"))
            .title("New payment received");
        p.state().sender.send(payload).await?;
    } else {
        log::error!("ntfy 'topic' not found");
    }

    Ok(())
}
