use entity_derive::Entity;

use crate::{entities::reclaim::simple_stats::SimpleStats, shared::{colliders::ComplexCollider, death_accounting::DependSet, health::Health, position::Position, simple_health::SimpleHealth, simple_mesh_info::SimpleMeshInfo, simple_position::SimplePos}, turbo_game_engine::TurboGameEngineTID};

use hord3::{defaults::default_rendering::{mantle::api::{CPUInstanceData, MantleHandler}, vectorinator::{VectorinatorWrite, meshes::{MeshID, MeshInstance}}}, horde::{game_engine::{entity::{ChosenID, Component, ComponentEvent, EVecStopsIn, EVecStopsOut, Entity, EntityID, EntityIDAllocator, EntityVec, MultiplayerEntity, NewEntity, Renderable, StaticComponent, StaticEntity}, multiplayer::{Identify, MustSync}, position::EntityPosition, static_type_id::HasStaticTypeID}, geometry::{rotation::{Orientation, Rotation}, vec3d::Vec3Df}, utils::ARW}};

pub mod simple_stats;

#[derive(Entity, Clone)]
pub struct Reclaim {
    #[used_in_new]
    #[used_in_render]
    #[must_sync]
    #[position]
    pos:SimplePos,
    #[used_in_new]
    #[static_id]
    stats:SimpleStats,
    health:SimpleHealth,
    collider:ComplexCollider,
    depend:DependSet<TurboGameEngineTID>,
    #[used_in_render]
    #[no_sync]
    mesh_info:SimpleMeshInfo
}

impl<ID:Identify> RenderReclaim<MantleHandler, ID> for Reclaim {
    fn do_render_changes(rendering_data: &mut MantleHandler, pos: &mut SimplePos, stats: &mut SimpleStats, mesh_info:&mut SimpleMeshInfo, static_type: &StaticReclaim<ID>) {
        match mesh_info.instance_id {
            Some(id) => {
                rendering_data.update_instance_no_rep(static_type.mesh_info.mesh_id.clone(), id, CPUInstanceData {
                    position:pos.pos,
                    speed:Vec3Df::zero(),
                    rotation:Orientation::zero()
                });
            },
            None => {
                if rendering_data.get_mesh(static_type.mesh_info.mesh_id.clone()).is_none() {
                    rendering_data.set_or_add_mesh_no_rep(static_type.mesh_info.mesh_id.unwrap_name(),static_type.mesh_info.mesh_data.clone(), Vec::new());
                }
                else {
                    mesh_info.instance_id = Some(rendering_data.add_instance_no_rep(static_type.mesh_info.mesh_id.clone(), CPUInstanceData {
                        position:pos.pos,
                        speed:Vec3Df::zero(),
                        rotation:Orientation::zero()
                    }))
                }
            }
        }
    }
}

impl<ID:Identify> NewEntity<Reclaim, ID> for NewReclaim<ID> {
    fn get_ent(self, static_type:&<Reclaim as Entity<ID>>::SE) -> Reclaim {
        Reclaim { pos: self.pos, stats: self.stats, collider:<ComplexCollider as Component<ID>>::from_static(&static_type.collider), mesh_info:SimpleMeshInfo::new(), health: <SimpleHealth as Component<ID>>::from_static(&static_type.health), depend: <DependSet<TurboGameEngineTID> as Component<ID>>::from_static(&static_type.depend) }
    }
} 