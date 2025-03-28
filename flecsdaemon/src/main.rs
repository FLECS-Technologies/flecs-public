use async_signal::{Signal, Signals};
use flecs_core::fsm::world::FlecsWorld;
use futures_util::StreamExt;
use tracing::info;

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
    flecs_core::fsm::init_tracing();
    let stop_signal = init_signal_handler()?;
    let world = FlecsWorld::create_default().await?;
    stop_signal.await?;
    world.halt().await;
    Ok(())
}
