use hord3::{defaults::default_rendering::mantle::{api::{ApiLod, MantleHandler}, meshes::MeshID}, horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify}};
use to_from_bytes_derive::{FromBytes, ToBytes};


#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub struct SimpleMeshInfo {
    pub instance_id:Option<usize>,

}

impl SimpleMeshInfo {
    pub fn free_mesh(&self, static_mesh:&StaticSimpleMeshInfo, mantle:&MantleHandler) {

        match self.instance_id {
            Some(instance) => {
                mantle.free_instance_no_rep(static_mesh.mesh_id.clone(), instance);
            },
            None => ()
        }
    }
}

impl SimpleMeshInfo {
    pub fn new() -> Self {
        Self { instance_id: None }
    }
}

#[derive(Clone)]
pub struct StaticSimpleMeshInfo {
    pub mesh_data:Vec<ApiLod>,
    pub mesh_id:MeshID,
}

impl StaticComponent for StaticSimpleMeshInfo {

}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum SimpleMeshUpdate {
    UpdateInstanceID(Option<usize>)
}

impl<ID:Identify> SimpleComponentUpdate<SimpleMeshInfo, ID> for SimpleMeshUpdate {
    fn apply_to_comp(self, component:&mut SimpleMeshInfo) {
        match self {
            SimpleMeshUpdate::UpdateInstanceID(id) => component.instance_id = id
        }
    }
}

pub type SimpleMeshEvent<ID:Identify> = SimpleComponentEvent<ID, SimpleMeshUpdate>;

impl<ID:Identify> Component<ID> for SimpleMeshInfo {
    type SC = StaticSimpleMeshInfo;
    type CE = SimpleMeshEvent<ID>;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { instance_id: None }
    }
}