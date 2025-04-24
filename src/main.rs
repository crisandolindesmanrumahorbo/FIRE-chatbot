use stockbit_chatbot::server::Server;
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::oneshot,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let server_handle = tokio::spawn(async move {
        let _ = Server::start(shutdown_rx).await;
    });

    gracefully_shutdown(shutdown_tx, server_handle).await;

    Ok(())
}

async fn gracefully_shutdown(
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    server_handle: tokio::task::JoinHandle<()>,
) {
    let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
    let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = signal_terminate.recv() => {
            println!("Shutdown signal_terminate received");
        },
        _ = signal_interrupt.recv() => {
                println!("Shutdown signal_interrupt received");
            }
    }
    let _ = shutdown_tx.send(());
    let _ = server_handle.await;
    println!("shutdown completed");
}
