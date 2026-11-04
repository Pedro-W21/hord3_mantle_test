use entity_derive::Entity;

use crate::{entities::reclaim::simple_stats::SimpleStats, shared::{colliders::ComplexCollider, death_accounting::DependSet, health::Health, position::Position, resources::TeamResources, simple_health::SimpleHealth, simple_position::SimplePos}, turbo_game_engine::TurboGameEngineTID};

use hord3::{defaults::default_rendering::{mantle::api::MantleHandler, vectorinator::{VectorinatorWrite, meshes::{MeshID, MeshInstance}}}, horde::{game_engine::{entity::{ChosenID, Component, ComponentEvent, EVecStopsIn, EVecStopsOut, Entity, EntityID, EntityIDAllocator, EntityVec, MultiplayerEntity, NewEntity, Renderable, StaticComponent, StaticEntity}, multiplayer::{Identify, MustSync}, position::EntityPosition, static_type_id::HasStaticTypeID}, geometry::{rotation::{Orientation, Rotation}, vec3d::Vec3Df}, utils::ARW}};

pub mod static_teams;

#[derive(Entity, Clone)]
pub struct Team {
    #[position]
    pos:SimplePos,
    #[used_in_new]
    #[static_id]
    #[must_sync]
    stats:SimpleStats,
    depend:DependSet<TurboGameEngineTID>,
    #[must_sync]
    resources:TeamResources
}

impl<ID:Identify> NewEntity<Team, ID> for NewTeam<ID> {
    fn get_ent(self, static_type:&<Team as Entity<ID>>::SE) -> Team {
        Team { pos: SimplePos {pos:Vec3Df::zero()}, stats: self.stats, depend: <DependSet<TurboGameEngineTID> as Component<ID>>::from_static(&static_type.depend), resources:<TeamResources as Component<ID>>::from_static(&static_type.resources) }
    }
} 