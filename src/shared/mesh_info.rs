use hord3::{defaults::default_rendering::mantle::{api::{ApiLod, MantleHandler}, meshes::MeshID}, horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::shared::turrets::StaticTurrets;


#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub struct MeshInfo {
    pub main_instance_id:Option<usize>,
    pub turret_instances:Vec<Option<usize>>
}

impl MeshInfo {
    pub fn new() -> Self {
        Self { main_instance_id: None, turret_instances: Vec::new() }
    }
    pub fn free_meshes(&self, static_meshes:&StaticMeshInfo, static_turrets:&StaticTurrets, mantle:&MantleHandler) {
        match self.main_instance_id {
            Some(instance) => {
                mantle.free_instance_no_rep(static_meshes.mesh_id.clone(), instance);
            },
            None => ()
        }
        let mut turret_index = 0;
        for turret in &static_turrets.turrets {
            for pos in &turret.positions {
                match self.turret_instances[turret_index] {
                    Some(instance) => {
                        mantle.free_instance_no_rep(turret.mesh_info.mesh_id.clone(), instance);
                    },
                    None => ()
                }
                turret_index += 1;
            }
        }
    }
}

#[derive(Clone)]
pub struct StaticMeshInfo {
    pub mesh_data:Vec<ApiLod>,
    pub mesh_id:MeshID,
}

impl StaticComponent for StaticMeshInfo {

}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum MeshUpdate {
    UpdateInstanceID(Option<usize>),
    UpdateTurretInstanceIDs(Vec<Option<usize>>)
}

impl<ID:Identify> SimpleComponentUpdate<MeshInfo, ID> for MeshUpdate {
    fn apply_to_comp(self, component:&mut MeshInfo) {
        match self {
            MeshUpdate::UpdateInstanceID(id) => component.main_instance_id = id,
            MeshUpdate::UpdateTurretInstanceIDs(ids) => component.turret_instances = ids,
        }
    }
}

pub type MeshEvent<ID:Identify> = SimpleComponentEvent<ID, MeshUpdate>;

impl<ID:Identify> Component<ID> for MeshInfo {
    type SC = StaticMeshInfo;
    type CE = MeshEvent<ID>;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { main_instance_id: None, turret_instances:Vec::new() }
    }
}