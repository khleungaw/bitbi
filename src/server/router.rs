use std::collections::HashMap;
use std::net::SocketAddr;

use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use log::info;
use tokio_tungstenite::tungstenite::Message;

pub enum RouterCommand
{
    Add(SocketAddr, UnboundedSender<Message>),
    Remove(SocketAddr),
    Forward(SocketAddr, Message),
    Broadcast(Message),
}

pub struct Router
{
    pub connections: HashMap<SocketAddr, UnboundedSender<Message>>,
    pub router_rx: UnboundedReceiver<RouterCommand>,
}

impl Router
{
    pub async fn run(&mut self)
    {
        info!("Starting");
        while let Ok(cmd) = self.router_rx.recv().await
        {
            match cmd
            {
                RouterCommand::Add(addr, connection) => self.handle_add(addr, connection),
                RouterCommand::Remove(addr) => self.handle_remove(&addr),
                RouterCommand::Forward(addr, msg) => self.handle_forward(&addr, msg),
                RouterCommand::Broadcast(msg) => self.handle_broadcast(msg),
            }
        }
    }

    fn handle_add(&mut self, addr: SocketAddr, connection: UnboundedSender<Message>)
    {
        info!("Registering: {}", addr);
        self.connections.insert(addr, connection);
    }

    fn handle_remove(&mut self, addr: &SocketAddr)
    {
        self.connections.remove(addr);
    }

    fn handle_forward(&mut self, addr: &SocketAddr, msg: Message)
    {
        if let Some(tx) = self.connections.get(addr)
            && tx.unbounded_send(msg).is_err()
        {
            self.connections.remove(addr);
        }
    }

    fn handle_broadcast(&mut self, msg: Message)
    {
        info!("Broadcasting");
        for connection in self.connections.values()
        {
            let _ = connection.unbounded_send(msg.clone());
        }
    }
}
