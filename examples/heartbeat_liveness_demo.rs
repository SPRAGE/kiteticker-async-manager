use kiteticker_async_manager::{KiteTickerManagerBuilder, Mode};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();

  let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
  if api_key.is_empty() || access_token.is_empty() {
    eprintln!(
      "Set KITE_API_KEY and KITE_ACCESS_TOKEN env vars to run this example."
    );
    return Ok(());
  }

  // Configure a custom heartbeat liveness threshold (e.g., 15 seconds)
  let mut manager = KiteTickerManagerBuilder::new(api_key, access_token)
    .max_connections(1)
    .default_mode(Mode::Quote)
    .enable_dedicated_parsers(true)
    .heartbeat_liveness_threshold(Duration::from_secs(15))
    .build();

  manager.start().await?;

  // Subscribe to a quiet symbol to see heartbeats keep the connection alive
  // Replace with a token valid for your account/segment
  manager
    .subscribe_symbols(&[256265], Some(Mode::Quote))
    .await?;

  // Periodically print health summary
  for _ in 0..10 {
    match manager.get_health().await {
      Ok(summary) => {
        println!(
          "Health: healthy={} degraded={} critical={} uptime={:?}",
          summary.is_healthy(),
          summary.is_degraded(),
          summary.is_critical(),
          summary.uptime
        );
      }
      Err(e) => eprintln!("Health error: {}", e),
    }
    tokio::time::sleep(Duration::from_secs(3)).await;
  }

  manager.stop().await?;
  Ok(())
}
