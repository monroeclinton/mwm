use actix::{Actor, Context, Handler, Message, Supervised, SystemService};
use anyhow::Result;

#[derive(Clone, PartialEq)]
pub struct Client {
    pub window: xcb::Window,
    pub visible: bool,
}

#[derive(Default)]
pub struct Clients {
    pub clients: Vec<Client>,
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
        // There won't be many clients, so this isn't completely horrible.
        // Vec is easier for actors to handle compared to VecDeque
        // because MessageResponse is implemented for Vec.
        self.clients.insert(0, Client {
            window: msg.window,
            visible: true,
        });

        Ok(())
    }
}

pub struct GetClients;

impl Message for GetClients {
    type Result = Vec<Client>;
}

impl Handler<GetClients> for Clients {
    type Result = Vec<Client>;

    fn handle(&mut self, _msg: GetClients, _ctx: &mut Self::Context) -> Self::Result {
        self.clients.clone()
    }
}
