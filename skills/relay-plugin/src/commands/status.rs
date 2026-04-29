use clap::Args;
use crate::api::get_status;

#[derive(Args)]
pub struct StatusArgs {
    /// Request ID from a previous bridge command
    #[arg(long)]
    pub request_id: String,
}

pub async fn run(args: StatusArgs) -> anyhow::Result<()> {
    let status = get_status(&args.request_id).await?;

    let dest_tx = status.tx_hashes.as_ref()
        .and_then(|txs| txs.iter().find(|t| t.is_destination_tx == Some(true)))
        .and_then(|t| t.tx_hash.as_deref());
    let origin_tx = status.tx_hashes.as_ref()
        .and_then(|txs| txs.iter().find(|t| t.is_destination_tx != Some(true)))
        .and_then(|t| t.tx_hash.as_deref());

    let out = serde_json::json!({
        "status":      status.status,
        "request_id":  args.request_id,
        "origin_tx":   origin_tx.unwrap_or("pending"),
        "dest_tx":     dest_tx.unwrap_or("pending"),
        "error":       status.error.as_deref().unwrap_or(""),
    });

    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
