use ferrumc_macros::{packet, Decode};

use crate::net::packets::IncomingPacket;
use crate::state::GlobalState;
use crate::utils::components::rotation::Rotation;
use crate::utils::encoding::position::Position;
use crate::utils::prelude::*;
use crate::Connection;

#[derive(Decode)]
#[packet(packet_id = 0x15, state = "play")]
pub struct SetPlayerPosAndRotate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl IncomingPacket for SetPlayerPosAndRotate {
    async fn handle(self, conn: &mut Connection, state: GlobalState) -> Result<()> {
        let my_entity_id = conn.id;

        let component_storage = state.world.get_component_storage();

        let mut position = component_storage
            .get_mut::<Position>(my_entity_id)
            .await
            .ok_or(Error::from(crate::ecs::error::Error::ComponentNotFound))?;
        let mut rotation = component_storage
            .get_mut::<Rotation>(my_entity_id)
            .await
            .ok_or(Error::from(crate::ecs::error::Error::ComponentNotFound))?;

        *position = Position {
            x: self.x as i32,
            y: self.y as i16,
            z: self.z as i32,
        };

        *rotation = Rotation {
            yaw: self.yaw,
            pitch: self.pitch,
        };

        Ok(())
    }
}
