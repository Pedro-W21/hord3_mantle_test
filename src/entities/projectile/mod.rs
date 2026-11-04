use entity_derive::Entity;

use crate::{entities::projectile::projectile_stats::ProjectileStats, shared::{colliders::ComplexCollider, death_accounting::DependSet, health::Health, position::Position, simple_health::SimpleHealth, simple_mesh_info::SimpleMeshInfo}, turbo_game_engine::TurboGameEngineTID};

use hord3::{defaults::default_rendering::{mantle::api::{CPUInstanceData, MantleHandler}, vectorinator::{VectorinatorWrite, meshes::{MeshID, MeshInstance}}}, horde::{game_engine::{entity::{ChosenID, Component, ComponentEvent, EVecStopsIn, EVecStopsOut, Entity, EntityID, EntityIDAllocator, EntityVec, MultiplayerEntity, NewEntity, Renderable, StaticComponent, StaticEntity}, multiplayer::{Identify, MustSync}, position::EntityPosition, static_type_id::HasStaticTypeID}, geometry::{rotation::{Orientation, Rotation}, vec3d::Vec3Df}, utils::ARW}};

pub mod projectile_stats;
pub mod static_projectiles;

#[derive(Entity, Clone)]
pub struct Projectile {
    #[used_in_new]
    #[used_in_render]
    #[must_sync]
    #[position]
    pos:Position,
    #[used_in_new]
    #[static_id]
    stats:ProjectileStats,
    collider:ComplexCollider,
    health:SimpleHealth,
    depend:DependSet<TurboGameEngineTID>,
    #[used_in_render]
    #[no_sync]
    mesh_info:SimpleMeshInfo
}

impl<ID:Identify> RenderProjectile<MantleHandler, ID> for Projectile {
    fn do_render_changes(rendering_data: &mut MantleHandler, pos: &mut Position, stats: &mut ProjectileStats, mesh_info: &mut SimpleMeshInfo, static_type: &StaticProjectile<ID>) {
        match mesh_info.instance_id {
            Some(id) => {
                rendering_data.update_instance_no_rep(static_type.mesh_info.mesh_id.clone(), id, CPUInstanceData {
                    position:pos.pos,
                    speed:pos.spd,
                    rotation:Rotation::from_orientation(Orientation::zero())
                });
            },
            None => {
                if rendering_data.get_mesh(static_type.mesh_info.mesh_id.clone()).is_none() {
                    rendering_data.set_or_add_mesh_no_rep(static_type.mesh_info.mesh_id.unwrap_name(),static_type.mesh_info.mesh_data.clone(), Vec::new());
                }
                else {
                    mesh_info.instance_id = Some(rendering_data.add_instance_no_rep(static_type.mesh_info.mesh_id.clone(), CPUInstanceData {
                        position:pos.pos,
                        speed:pos.spd,
                        rotation:Rotation::from_orientation(Orientation::zero())
                    }))
                }
            }
        }
    }
}

impl<ID:Identify> NewEntity<Projectile, ID> for NewProjectile<ID> {
    fn get_ent(self, static_type:&<Projectile as Entity<ID>>::SE) -> Projectile {
        let pos = self.pos.pos;
        Projectile { pos: self.pos, stats: self.stats, mesh_info:SimpleMeshInfo::new(), collider: static_type.collider.clone(), health: <SimpleHealth as Component<ID>>::from_static(&static_type.health), depend: <DependSet<TurboGameEngineTID> as Component<ID>>::from_static(&static_type.depend) }
    }
} 