use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::UnboundedSender;
use log::info;
use tokio_tungstenite::tungstenite::Message;

use crate::server::Balance;
use crate::server::RouterCommand;

pub struct Broadcast
{
    pub router_tx: UnboundedSender<RouterCommand>,
    pub balance_rx: Receiver<Balance>,
}

impl Broadcast
{
    pub async fn run(&mut self)
    {
        info!("Starting");
        while let Ok(balance) = self.balance_rx.recv().await
        {
            let json: String = serde_json::to_string(&balance).unwrap_or("{}".to_string());
            let msg = Message::text(json);
            let _ = self.router_tx.unbounded_send(RouterCommand::Broadcast(msg));
        }
    }
}
