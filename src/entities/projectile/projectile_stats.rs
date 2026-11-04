use hord3::horde::{game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify, static_type_id::HasStaticTypeID}, geometry::vec3d::Vec3Df};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::shared::{attack::Damage, resources::ResourceCost, targeting::TargetEnvironment};

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub struct ProjectileStats {
    pub team:u16,
    pub end_tick:usize,
    static_type_id:usize,
    pub dead:bool
}

impl ProjectileStats {
    pub fn new(team:u16, ttl:usize, start_tick:usize, static_type_id:usize) -> Self {
        Self { team, end_tick:ttl + start_tick, static_type_id, dead:false }
    }
    
    pub fn ttl_finished(&self, current_tick:usize) -> bool {
        current_tick >= self.end_tick
    }
}

impl HasStaticTypeID for ProjectileStats {
    fn get_id(&self) -> usize {
        self.static_type_id
    }
}

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub enum ProjectileStatsUpdate {
    UpgradeProgess(f32),
    Kill
}

impl<ID:Identify> SimpleComponentUpdate<ProjectileStats, ID> for ProjectileStatsUpdate {
    fn apply_to_comp(self, component:&mut ProjectileStats) {
        match self {
            Self::UpgradeProgess(_) => (),
            Self::Kill => component.dead = true
        }
    }
}

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub struct StaticProjectileStats {
    pub upgrade_speed:f32,
    pub on_death:ProjectileDeath,
    pub aoe:ProjectileAOE,
    pub gravity_felt:Vec3Df,
    pub max_speed:f32,
    pub target_environments:Vec<TargetEnvironment>,
    pub ttl:usize,
}

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub enum ProjectileDeath {
    Simple,
    SummonProjectile,
    DoAOE(ProjectileAOE)
}

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub enum ProjectileAOE {
    NoAOE,
    DirectDamage {range:f32, damage:Damage}
}

impl StaticComponent for StaticProjectileStats {

}

pub type ProjectileStatsEvent<ID:Identify> = SimpleComponentEvent<ID, ProjectileStatsUpdate>;

impl<ID:Identify> Component<ID> for ProjectileStats {
    type CE = SimpleComponentEvent<ID, ProjectileStatsUpdate>;
    type SC = StaticProjectileStats;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self::new(0, 0, 0, 0)
    }
}


