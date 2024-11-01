use ferrumc_ecs::Universe;
use ferrumc_macros::bake_packet_registry;
use std::sync::{Arc};
use ferrumc_world::World;

pub mod connection;
pub mod errors;
pub mod packets;
pub mod server;
pub mod utils;
pub type NetResult<T> = Result<T, errors::NetError>;

pub struct ServerState {
    pub universe: Universe,
    pub minecraft_world: World
}

pub type GlobalState = Arc<ServerState>;

impl ServerState {
    pub fn new(universe: Universe, minecraft_world: World) -> Self {
        Self {
            universe,
            minecraft_world
        }
    }
}

bake_packet_registry!("\\src\\packets\\incoming");
