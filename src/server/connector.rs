use futures::StreamExt;
use futures::TryStreamExt;
use futures::channel::mpsc::UnboundedSender;
use futures::channel::mpsc::unbounded;
use futures::future;
use std::net::SocketAddr;
use std::pin::pin;
use std::sync::Arc;

use log::debug;
use log::info;
use log::warn;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

use crate::server::LedgerCommand;
use crate::server::message_parser::parse_message;
use crate::server::router::RouterCommand;
use crate::settings::Settings;

pub struct Connector
{
    pub settings: Arc<Settings>,
    pub router_tx: UnboundedSender<RouterCommand>,
    pub ledger_tx: UnboundedSender<LedgerCommand>,
}

impl Connector
{
    pub async fn run(&mut self)
    {
        info!("Starting");
        let addr = format!("{}:{}", self.settings.addr, self.settings.port);
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

        while let Ok((stream, addr)) = listener.accept().await
        {
            tokio::spawn(Self::handle_connection(
                self.router_tx.clone(),
                self.ledger_tx.clone(),
                stream,
                addr,
            ));
        }
    }

    async fn handle_connection(
        router_tx: UnboundedSender<RouterCommand>,
        ledger_tx: UnboundedSender<LedgerCommand>,
        stream: TcpStream,
        addr: SocketAddr,
    )
    {
        debug!("Incoming TCP connection from: {}", addr);
        let ws_stream = match tokio_tungstenite::accept_async(stream).await
        {
            Ok(ws) => ws,
            Err(e) =>
            {
                warn!("Handshake failed for {}: {}", addr, e);
                return;
            }
        };

        debug!("WebSocket connection established: {}", addr);
        let (write, read) = ws_stream.split();
        let (tx, rx) = unbounded::<Message>();
        let _ = router_tx.unbounded_send(RouterCommand::Add(addr, tx));

        // Client -> Server: read -> ledger_tx
        let read_handler = read.try_for_each(|msg| {
            debug!("Client {}: {}", addr, msg);
            let _ = ledger_tx.unbounded_send(parse_message(addr, msg));
            future::ok(())
        });

        // Server -> Client: router_tx -> tx -> rx -> write
        let write_handler = rx.map(Ok).forward(write);
        let read_handler = pin!(read_handler);
        let write_handler = pin!(write_handler);
        future::select(read_handler, write_handler).await;

        // Cleanup
        let _ = router_tx.unbounded_send(RouterCommand::Remove(addr));
        debug!("Connection closed: {}", addr);
    }
}
