use core::time::Duration;
use futures::try_join;
use tokio::time::sleep;
use crate::types::*;
use crate::stats::*;

pub async fn stat_worker(ctx: Arc<Context>) -> Result<()> {
    loop {
        sleep(Duration::from_secs(64)).await;
        let mut st = ctx.stats();
        st.tick();
        save_stats(&st).await;
    }
}

pub async fn run_worker(ctx: Arc<Context>) -> Result<()> {
    let sw = stat_worker(ctx.clone());

    try_join!(sw)?;
    Ok(())
}
