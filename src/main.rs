// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use cln_plugin::options::{ConfigOption, Value};
use cln_plugin::{Builder, Error, Plugin};
use ntfy::{Auth, Dispatcher, Payload};

#[derive(Clone)]
struct PluginState {
    dispatcher: Dispatcher,
    topic: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let plugin = match Builder::new(tokio::io::stdin(), tokio::io::stdout())
        .option(ConfigOption::new(
            "ntfy-url",
            Value::String(String::new()),
            "ntfy url",
        ))
        .option(ConfigOption::new(
            "ntfy-topic",
            Value::String(String::from("cln-alerts")),
            "ntfy topic",
        ))
        .option(ConfigOption::new(
            "ntfy-username",
            Value::OptString,
            "ntfy username",
        ))
        .option(ConfigOption::new(
            "ntfy-password",
            Value::OptString,
            "ntfy password",
        ))
        .option(ConfigOption::new(
            "ntfy-proxy",
            Value::OptString,
            "ntfy proxy",
        ))
        .subscribe("invoice_payment", invoice_payment_handler)
        .configure()
        .await?
    {
        Some(p) => p,
        None => return Ok(()),
    };

    let url: String = match plugin.option("ntfy-url") {
        Some(Value::String(url)) => {
            if url.is_empty() {
                panic!("ntfy-url required")
            } else {
                url
            }
        }
        _ => panic!("ntfy-url required"),
    };

    let topic: String = match plugin.option("ntfy-topic") {
        Some(Value::String(topic)) => {
            if topic.is_empty() {
                panic!("ntfy-topic required")
            } else {
                topic
            }
        }
        _ => panic!("ntfy-topic required"),
    };

    let mut dispatcher = Dispatcher::builder(url);

    if let Some(Value::String(username)) = plugin.option("ntfy-username") {
        if let Some(Value::String(password)) = plugin.option("ntfy-password") {
            dispatcher = dispatcher.credentials(Auth::new(username, password));
        } else {
            log::error!("ntfy-password missing");
        }
    }

    if let Some(Value::String(proxy)) = plugin.option("ntfy-proxy") {
        dispatcher = dispatcher.proxy(proxy);
    }

    let state = PluginState {
        dispatcher: dispatcher.build()?,
        topic,
    };
    let plugin = plugin.start(state).await?;

    plugin.join().await?;

    Ok(())
}

async fn invoice_payment_handler(
    p: Plugin<PluginState>,
    v: serde_json::Value,
) -> Result<(), Error> {
    log::debug!("Got a invoice payment notification: {v}");
    let amount: u64 = extract_amount(v)?;
    let payload = Payload::new(&p.state().topic)
        .message(format!("+{amount} sat"))
        .title("New payment received")
        .tags(vec!["zap"]);
    p.state().dispatcher.send(&payload).await?;
    Ok(())
}

fn extract_amount(value: serde_json::Value) -> Result<u64, Error> {
    let amount_msat = &value["invoice_payment"]["msat"];
    let amount_str: String = serde_json::from_value(amount_msat.clone())?;
    let amount: usize = amount_str[..amount_str.len() - 4].parse()?;
    Ok((amount / 1000) as u64)
}
