use std::collections::HashMap;
use std::net::SocketAddr;

use chrono::NaiveDate;
use futures::channel::mpsc::Sender;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use log::debug;
use log::error;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::calc::Cent;
use crate::server::router::RouterCommand;

pub enum LedgerCommand
{
    Add(SocketAddr, LedgerEntry),
    Delete(SocketAddr, Uuid),
}

#[derive(Debug, Deserialize)]
pub struct LedgerEntry
{
    pub value_date: NaiveDate,
    pub account: String,
    pub amt: Cent,
    pub ccy: String,
}

pub type Balance = HashMap<String, Cent>;

pub struct Ledger
{
    pub entries: HashMap<Uuid, LedgerEntry>,
    pub balances: HashMap<String, Cent>,
    pub router_tx: UnboundedSender<RouterCommand>,
    pub balance_tx: Sender<Balance>,
    pub ledger_rx: UnboundedReceiver<LedgerCommand>,
}

impl Ledger
{
    pub async fn run(&mut self)
    {
        info!("Starting");
        while let Ok(cmd) = self.ledger_rx.recv().await
        {
            match cmd
            {
                LedgerCommand::Add(addr, entry) => self.handle_add(addr, entry).await,
                LedgerCommand::Delete(addr, id) => self.handle_delete(addr, id).await,
            }
        }
    }

    fn compute_balance(&mut self)
    {
        self.balances.clear();
        for entry in self.entries.values()
        {
            let current = self.balances.get(&entry.account).copied().unwrap_or(0);
            self.balances
                .insert(entry.account.clone(), current + entry.amt);
        }
    }

    async fn handle_add(&mut self, addr: SocketAddr, entry: LedgerEntry)
    {
        info!("Adding {:?}", entry);
        let id = Uuid::new_v4();
        self.entries.insert(id, entry);
        self.compute_balance();
        self.respond(addr, id.to_string());
        self.publish_balance().await;
    }

    async fn handle_delete(&mut self, addr: SocketAddr, id: Uuid)
    {
        info!("Deleting {}", id);
        self.entries.remove(&id);
        self.compute_balance();
        self.respond(addr, id.to_string());
        self.publish_balance().await;
    }

    fn respond<T: Serialize>(&mut self, addr: SocketAddr, content: T)
    {
        debug!("Responding");
        let json = serde_json::to_string(&content).unwrap_or("{}".to_string());
        let msg = Message::text(json.to_string());
        let cmd = RouterCommand::Forward(addr, msg);
        let _ = self.router_tx.unbounded_send(cmd);
    }

    async fn publish_balance(&mut self)
    {
        let _ = self
            .balance_tx
            .send(self.balances.clone())
            .await
            .inspect_err(|e| error!("Failed to write to Balance: {e}"));
    }
}
