use async_signal::{Signal, Signals};
use flecs_core::fsm::world::FlecsWorld;
use futures_util::StreamExt;
use std::sync::Arc;
use tracing::{info, trace};

fn init_signal_handler() -> std::io::Result<tokio::sync::oneshot::Receiver<()>> {
    let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
    let mut signals = Signals::new([Signal::Term, Signal::Int])?;
    tokio::spawn(async move {
        info!("Signal handler was initialized");
        while let Some(signal) = signals.next().await {
            info!("Received signal {signal:?}");
            if matches!(signal, Ok(Signal::Int) | Ok(Signal::Term)) {
                result_sender.send(()).unwrap();
                break;
            }
        }
    });
    Ok(result_receiver)
}

#[tokio::main]
async fn main() -> flecs_core::fsm::Result<()> {
    flecs_core::fsm::init_backtracing();
    let lore = FlecsWorld::read_lore().await?;
    flecs_core::fsm::init_tracing(&lore.tracing_filter);
    trace!("Using {lore:#?}");
    let stop_signal = init_signal_handler()?;
    let world = if std::env::args().any(|arg| &arg == "--migrate")
        || FlecsWorld::migration_necessary().await
    {
        FlecsWorld::migrate(Arc::new(lore)).await?
    } else {
        FlecsWorld::create_from_config(Arc::new(lore)).await?
    };
    stop_signal.await?;
    world.halt().await;
    Ok(())
}
