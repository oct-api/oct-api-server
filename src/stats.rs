use crate::stor::Entry;
use crate::types::*;

fn stats_file() -> Entry {
    Entry::new("stat.json")
}

pub async fn try_load_stats() -> Stats {
    let newstat = Stats::new();
    match stats_file().read().await {
        Ok(x) => serde_json::from_str(&x).unwrap_or(newstat),
        _ => newstat,
    }
}

pub async fn save_stats(stats: &Stats) -> Result<()> {
    stats_file().write_json(stats).await
}
