use std::collections::HashMap;
use std::net::SocketAddr;

use chrono::NaiveDate;
use futures::channel::mpsc::Sender;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use log::info;
use uuid::Uuid;

use crate::calc::Cent;
use crate::server::router::RouterCommand;

pub enum LedgerCommand
{
    Add(SocketAddr, LedgerEntry),
    Delete(SocketAddr, Uuid),
}

#[derive(Debug)]
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
                LedgerCommand::Delete(addr, id) => self.handle_delete(addr, id),
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
        self.forward_balance().await;
    }

    fn handle_delete(&mut self, addr: SocketAddr, id: Uuid)
    {
        self.entries.remove(&id);
        self.compute_balance();
        self.forward_balance();
    }

    async fn forward_balance(&mut self)
    {
        self.balance_tx.send(self.balances.clone()).await;
    }
}
