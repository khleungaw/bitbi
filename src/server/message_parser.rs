use std::net::SocketAddr;

use chrono::NaiveDate;
use tokio_tungstenite::tungstenite::Message;

use crate::server::ledger::LedgerEntry;
use crate::server::LedgerCommand;

pub fn parse_message(addr: SocketAddr, msg: Message) -> LedgerCommand
{
    LedgerCommand::Add(
        addr,
        LedgerEntry {
            value_date: NaiveDate::from_ymd_opt(2026, 10, 3).unwrap(),
            account: "daniel".to_string(),
            amt: 1,
            ccy: "HKD".to_string(),
        },
    )
}
