use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "rust-eth-price", version, about = "Query ETH price via OKX API")]
struct Cli {
    /// Blockchain network
    #[arg(long, default_value = "ethereum")]
    chain: String,
}

#[derive(Deserialize)]
struct OkxResponse {
    data: Vec<OkxTicker>,
}

#[derive(Deserialize)]
struct OkxTicker {
    last: String,
    #[serde(rename = "open24h")]
    open_24h: String,
    #[serde(rename = "vol24h")]
    vol_24h: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let client = reqwest::Client::new();
    let resp: OkxResponse = client
        .get("https://www.okx.com/api/v5/market/ticker?instId=ETH-USDT")
        .send()
        .await?
        .json()
        .await?;

    if let Some(ticker) = resp.data.first() {
        let price: f64 = ticker.last.parse()?;
        let open: f64 = ticker.open_24h.parse()?;
        let change_pct = ((price - open) / open) * 100.0;

        let output = serde_json::json!({
            "chain": cli.chain,
            "token": "ETH",
            "price_usd": format!("{:.2}", price),
            "change_24h": format!("{:+.2}%", change_pct),
            "volume_24h": ticker.vol_24h,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        eprintln!("Error: No ticker data returned");
        std::process::exit(1);
    }

    Ok(())
}
