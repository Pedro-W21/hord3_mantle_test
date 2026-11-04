use hord3::horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify, static_type_id::HasStaticTypeID};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{entities::team::TeamVecRead, shared::{actions::TID, colliders::AABB, consts::RESOURCE_SHARE_SCALE, resources::{ResourceCost, ResourceProduction, ResourceStorage}, targeting::TargetEnvironment}};

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub struct Stats {
    pub team:u16,
    pub upgrade_progress:i32,
    static_type_id:usize,
    pub current_env:TargetEnvironment,
    pub dead:bool,
    pub resource_share:i64,
    pub resource_consumption:Option<ResourceCost>
}

impl Stats {
    pub fn new(team:u16, static_type_id:usize, current_env:TargetEnvironment, initial_upgrade:i32) -> Self {
        Self { team, upgrade_progress: initial_upgrade, static_type_id, current_env, dead:false, resource_share:RESOURCE_SHARE_SCALE, resource_consumption:None }
    }
    pub fn active(&self) -> bool {
        !self.dead && self.upgrade_progress <= 0
    }
    pub fn compute_share<'a>(&self, teams:&TeamVecRead<'a, TID>) -> i64 {
        match self.resource_consumption {
            Some(cons) => teams.resources[self.team as usize].get_share(cons.clone()),
            None => RESOURCE_SHARE_SCALE
        }
    }
}

impl HasStaticTypeID for Stats {
    fn get_id(&self) -> usize {
        self.static_type_id
    }
}

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub enum StatsUpdate {
    AddToUpgradeProgress(i32),
    Kill,
    UpdateShare(i64),
    UpdateConsumption(Option<ResourceCost>)
}

impl<ID:Identify> SimpleComponentUpdate<Stats, ID> for StatsUpdate {
    fn apply_to_comp(self, component:&mut Stats) {
        match self {
            Self::AddToUpgradeProgress(new_progress) => component.upgrade_progress += new_progress,
            Self::Kill => component.dead = true,
            Self::UpdateShare(share) => component.resource_share = share,
            Self::UpdateConsumption(cons) => component.resource_consumption = cons,
        }
    }
}

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub struct StaticStats {
    pub upgrade_speed:f32,
    pub build_contribution_cost:ResourceCost,
    pub repair_contribution_cost:ResourceCost,
    pub assist_contribution_cost:ResourceCost,
    pub resource_production:ResourceProduction,
    pub resource_storage:ResourceStorage,
    pub default_env:TargetEnvironment,
    pub construction_quantity:i32,
    pub building_footprint:AABB
}

impl StaticComponent for StaticStats {

}

pub type StatsEvent<ID:Identify> = SimpleComponentEvent<ID, StatsUpdate>;

impl<ID:Identify> Component<ID> for Stats {
    type CE = SimpleComponentEvent<ID, StatsUpdate>;
    type SC = StaticStats;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { team: 0, upgrade_progress: 0, static_type_id: 0, current_env:static_comp.default_env, dead:false, resource_share:RESOURCE_SHARE_SCALE, resource_consumption:None }
    }
}


