use std::net::SocketAddr;

use chrono::NaiveDate;
use log::{debug, info};
use serde::Deserialize;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::server::ledger::LedgerEntry;
use crate::server::LedgerCommand;

#[derive(Deserialize)]
pub enum ClientMessage
{
    Add(LedgerEntry),
    Delete(Uuid),
}

pub fn parse_message(addr: SocketAddr, msg: Message) -> Option<LedgerCommand>
{
    let text = msg
        .to_text()
        .inspect_err(|e| debug!("Failed to read message from {addr} as text: {e}"))
        .ok()?;

    info!("Request from {}: {}", addr, text);
    let parsed: ClientMessage = serde_json::from_str(text)
        .inspect_err(|e| debug!("Failed to parse message from {addr}: {e}"))
        .ok()?;

    match parsed
    {
        ClientMessage::Add(entry) => Some(LedgerCommand::Add(addr, entry)),
        ClientMessage::Delete(id) => Some(LedgerCommand::Delete(addr, id)),
    }
}
