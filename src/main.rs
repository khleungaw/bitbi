mod calc;
mod server;
mod settings;

use std::collections::HashMap;
use std::sync::Arc;

use futures::channel::mpsc;
use futures::channel::mpsc::unbounded;
use log::info;

use crate::server::Balance;
use crate::server::Broadcast;
use crate::server::Connector;
use crate::server::Ledger;
use crate::server::LedgerCommand;
use crate::server::Router;
use crate::server::RouterCommand;
use crate::settings::Settings;

#[tokio::main]
async fn main()
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let settings = Settings::new("local").unwrap();
    info!("{:?}", settings);
    let settings = Arc::new(settings);

    let (ledger_tx, ledger_rx) = unbounded::<LedgerCommand>();
    let (router_tx, router_rx) = unbounded::<RouterCommand>();
    let (balance_tx, balance_rx) = mpsc::channel::<Balance>(1);

    let mut connector = Connector {
        settings: settings.clone(),
        ledger_tx: ledger_tx.clone(),
        router_tx: router_tx.clone(),
    };

    let mut router = Router {
        connections: HashMap::new(),
        router_rx,
    };

    let mut ledger = Ledger {
        entries: HashMap::new(),
        balances: HashMap::new(),
        router_tx: router_tx.clone(),
        balance_tx: balance_tx.clone(),
        ledger_rx,
    };

    let mut broadcast = Broadcast {
        router_tx: router_tx.clone(),
        balance_rx,
    };

    let handle_server = tokio::spawn(async move { connector.run().await });
    let handle_router = tokio::spawn(async move { router.run().await });
    let handle_ledger = tokio::spawn(async move { ledger.run().await });
    let handle_broadcast = tokio::spawn(async move { broadcast.run().await });
    let _ = tokio::join!(
        handle_server,
        handle_router,
        handle_ledger,
        handle_broadcast
    );
}
