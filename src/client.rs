use std::collections::VecDeque;
use actix::{Actor, Context, Handler, Message, Supervised, SystemService};
use anyhow::Result;

#[derive(Clone, PartialEq)]
pub struct Client {
    pub window: xcb::Window,
    pub visible: bool,
}

#[derive(Default)]
pub struct Clients {
    clients: VecDeque<Client>,
}

impl Actor for Clients {
    type Context = Context<Self>;
}

impl Supervised for Clients {}
impl SystemService for Clients {}

pub struct CreateClient {
    pub window: xcb::Window,
}

impl Message for CreateClient {
    type Result = Result<()>;
}

impl Handler<CreateClient> for Clients {
    type Result = Result<()>;

    fn handle(&mut self, msg: CreateClient, _ctx: &mut Self::Context) -> Self::Result {
        self.clients.push_front(Client {
            window: msg.window,
            visible: true,
        });

        Ok(())
    }
}

pub struct GetClients;

impl Message for GetClients {
    type Result = Result<VecDeque<Client>>;
}

impl Handler<GetClients> for Clients {
    type Result = Result<VecDeque<Client>>;

    fn handle(&mut self, _msg: GetClients, _ctx: &mut Self::Context) -> Self::Result {
        Ok(self.clients.clone())
    }
}

pub async fn get_clients() -> Result<VecDeque<Client>> {
    Clients::from_registry()
        .send(GetClients)
        .await?
}
