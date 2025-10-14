// Copyright (c) 2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_graphql_client::{Client, error::Result};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[serde(default)]
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    id: u64,
    #[serde(default)]
    result: Option<T>,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[tokio::main]
async fn main() -> Result<()> {
    const HOST: &str = "https://graphql.iota-rebased-alphanet.iota.cafe";
    let client = Client::new(HOST).unwrap();
    let address = "0x44c911eb91fa52fb7e0eb517e095e24d0abdc61e7feca94ed4c47401e361d38f".parse()?;

    for coin in client
        .coins(address, None, Default::default())
        .await?
        .data()
    {
        println!("Coin = {}, Balance = {}", coin.id(), coin.balance());
    }

    let balance = client.balance(address, None).await?.unwrap_or_default();
    println!("Total balance = {balance}");

    match iotax_get_all_coins().await {
        Ok(coins) => println!("All coins: {:#}", coins),
        Err(e) => eprintln!("Error fetching all coins: {}", e),
    }

    Ok(())
}

/// Calls `iotax_getAllCoins` and returns the raw JSON `result`.
pub async fn iotax_get_all_coins() -> Result<Value, Box<dyn std::error::Error>> {
    let client = HttpClient::builder().build()?;
    let endpoint = "https://api.iota-rebased-alphanet.iota.cafe";
    let owner_addr = "0x44c911eb91fa52fb7e0eb517e095e24d0abdc61e7feca94ed4c47401e361d38f";

    // Build the JSON-RPC request
    let req = json!({
        "jsonrpc": "2.0",
        "id": 1u64,
        "method": "iotax_getAllCoins",
        "params": [owner_addr],
    });

    let resp = client
        .post(endpoint)
        .json(&req)
        .send()
        .await?
        .error_for_status()? // surface 4xx/5xx as errors
        .json::<JsonRpcResponse<Value>>()
        .await?;

    if let Some(err) = resp.error {
        Err(format!(
            "JSON-RPC error {}: {} ({:?})",
            err.code, err.message, err.data
        )
        .into())
    } else if let Some(result) = resp.result {
        Ok(result)
    } else {
        Err("JSON-RPC response missing both result and error".into())
    }
}
