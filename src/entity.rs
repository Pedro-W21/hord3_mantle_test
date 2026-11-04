use std::{collections::VecDeque, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}, time::Duration};

use entity_derive::{Entity};
use to_from_bytes_derive::{FromBytes, ToBytes};

use hord3::{defaults::default_rendering::vectorinator::{VectorinatorWrite, meshes::{MeshID, MeshInstance}}, horde::{game_engine::{entity::{ChosenID, Component, ComponentEvent, EVecStopsIn, EVecStopsOut, Entity, EntityID, EntityIDAllocator, EntityVec, MultiplayerEntity, NewEntity, Renderable, StaticComponent, StaticEntity}, multiplayer::{Identify, MustSync}, position::EntityPosition, static_type_id::HasStaticTypeID}, geometry::{rotation::{Orientation, Rotation}, vec3d::Vec3Df}, utils::ARW}};

#[derive(Clone, PartialEq, Eq, ToBytes, FromBytes)]
pub struct CoolComponent {
    pub pos:Vec3Df,
}

impl<ID:Identify> Component<ID> for CoolComponent {
    type SC = Self;
    type CE = Self;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { pos: static_comp.pos.clone() }
    }
}

impl<ID:Identify> EntityPosition<ID> for CoolComponent {
    fn get_pos(&self) -> Vec3Df {
        self.pos.clone()
    }
    fn get_orientation(&self) -> Orientation {
        Orientation::zero()
    }
    fn get_rotation(&self) -> Option<&hord3::horde::geometry::rotation::Rotation> {
        None
    }
}
#[derive(Clone, PartialEq, Eq, ToBytes, FromBytes)]
pub struct RenderingData {
    instance_id:Option<usize>,
}

impl<ID:Identify> Component<ID> for RenderingData {
    type CE = Self; 
    type SC = Self;
    fn from_static(static_comp:&Self::SC) -> Self {
        static_comp.clone()
    }
}
impl StaticComponent for RenderingData {

}
impl<ID:Identify> ComponentEvent<RenderingData, ID> for RenderingData {
    type ComponentUpdate = Self;
    fn get_id(&self) -> EntityID {
        0
    }
    fn apply_to_component(self, components:&mut Vec<RenderingData>) {
        let a = 2;
    }
    fn get_source(&self) -> Option<ID> {
        None
    }
}

impl HasStaticTypeID for CoolComponent {
    fn get_id(&self) -> usize {
        0
    }
}

impl StaticComponent for CoolComponent {
    
}

impl<ID:Identify> ComponentEvent<CoolComponent, ID> for CoolComponent {
    type ComponentUpdate = Self;
    fn get_id(&self) -> hord3::horde::game_engine::entity::EntityID {
        0
    }
    fn apply_to_component(self, components:&mut Vec<CoolComponent>) {
        let a = 1;
    }
    fn get_source(&self) -> Option<ID> {
        None
    }
}

#[derive(Entity, Clone)]

pub struct CoolEntity {
    #[used_in_new]
    #[used_in_render]
    #[must_sync]
    #[position]
    #[static_id]
    pub pos:CoolComponent,
    #[used_in_render]
    pub render:RenderingData,
}

impl<ID:Identify> NewEntity<CoolEntity, ID> for NewCoolEntity<ID> {
    fn get_ent(self, static_type:&StaticCoolEntity<ID>) -> CoolEntity {
        CoolEntity { pos: self.pos, render:RenderingData { instance_id: None } }
    }
}

impl<'a, ID:Identify> RenderCoolEntity<VectorinatorWrite<'a>, ID> for CoolEntity {
    fn do_render_changes(rendering_data:&mut VectorinatorWrite<'a>, pos: &mut CoolComponent, render:&mut RenderingData, static_type:&StaticCoolEntity<ID>) {
        match render.instance_id {
            Some(id) => {
                
            },
            None => {
                render.instance_id = Some(rendering_data.meshes.add_instance(
                    MeshInstance::new(pos.pos.clone(), Orientation::zero(), MeshID::Referenced(0), true, false, false)
                    , 2)
                )
            }
        }
    }
}

#[test]
fn test_stuff() {
    
}