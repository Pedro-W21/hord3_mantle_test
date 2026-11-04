use std::{collections::HashMap, hash::Hash};

use hord3::horde::{game_engine::{entity::SimpleComponentEvent, multiplayer::MustSync, static_type_id::HasStaticTypeID, world::WorldComputeHandler}, geometry::vec3d::{Vec3D, Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{entities::{building::{BuildingEvent, BuildingVecRead}, projectile::{ProjectileEvent, ProjectileVecRead}, reclaim::{ReclaimEvent, ReclaimVecRead}, team::{TeamEvent, TeamVecRead}, unit::{UnitEvent, UnitVecRead}}, game_engine::CoolVoxel, game_map::{GameMap, GridUsed, rts_gen::RTSGen}, shared::{actions::TID, colliders::AABB, consts::{AIR_SET, BUILDING_SET, GROUND_SET, NAVAL_SET, PROJECTILE_SET, RECLAIM_SET}, death_accounting::DependSetUpdate, health::HealthUpdate, resources::{TeamResourceEvent, TeamResourcesUpdate}, simple_health::SimpleHealthUpdate, stats::{Stats, StatsEvent, StatsUpdate}, turrets::TurretsUpdate}, turbo_game_engine::TurboGameEngineTID};

pub fn get_ground_units_in_range_with_same_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, units:&UnitVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, GROUND_SET, 1.0).filter(move |id| {
        units.stats[*id].team == team && position.dist_squared(&units.pos[*id].pos) <= range_squared
    })
}

pub fn get_air_units_in_range_with_same_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, units:&UnitVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, AIR_SET, 1.0).filter(move |id| {
        units.stats[*id].team == team && position.dist_squared(&units.pos[*id].pos) <= range_squared
    })
}

pub fn get_naval_units_in_range_with_same_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, units:&UnitVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, NAVAL_SET, 1.0).filter(move |id| {
        units.stats[*id].team == team && position.dist_squared(&units.pos[*id].pos) <= range_squared
    })
}

pub fn get_buildings_in_range_with_same_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, buildings:&BuildingVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, BUILDING_SET, 1.0).filter(move |id| {
        buildings.stats[*id].team == team && position.dist_squared(&buildings.pos[*id].pos) <= range_squared
    })
}
pub fn get_buildings_in_range<'a>(range:f32, position:Vec3Df, grid:&GridUsed, buildings:&BuildingVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, BUILDING_SET, 1.0).filter(move |id| {
        position.dist_squared(&buildings.pos[*id].pos) <= range_squared
    })
}


pub fn get_ground_units_in_range_with_diff_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, units:&UnitVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, GROUND_SET, 1.0).filter(move |id| {
        units.stats[*id].team != team && position.dist_squared(&units.pos[*id].pos) <= range_squared
    })
}

pub fn get_air_units_in_range_with_diff_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, units:&UnitVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, AIR_SET, 1.0).filter(move |id| {
        units.stats[*id].team != team && position.dist_squared(&units.pos[*id].pos) <= range_squared
    })
}

pub fn get_naval_units_in_range_with_diff_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, units:&UnitVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, NAVAL_SET, 1.0).filter(move |id| {
        units.stats[*id].team != team && position.dist_squared(&units.pos[*id].pos) <= range_squared
    })
}

pub fn get_buildings_in_range_with_diff_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, buildings:&BuildingVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, BUILDING_SET, 1.0).filter(move |id| {
        buildings.stats[*id].team != team && position.dist_squared(&buildings.pos[*id].pos) <= range_squared
    })
}

pub fn get_projectiles_in_range_with_diff_team<'a>(team:u16, range:f32, position:Vec3Df, grid:&GridUsed, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, PROJECTILE_SET, 1.0).filter(move |id| {
        projectiles.stats[*id].team != team && position.dist_squared(&projectiles.pos[*id].pos) <= range_squared
    })
}

pub fn get_reclaim_in_range<'a>(range:f32, position:Vec3Df, grid:&GridUsed, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) -> impl Iterator<Item = usize> {
    let range_squared = range * range;
    grid.get_iter_from_to(position - Vec3Df::all_ones() * range, position + Vec3Df::all_ones() * range, RECLAIM_SET, 1.0).filter(move |id| {
        position.dist_squared(&reclaim.pos[*id].pos) <= range_squared
    })
}

#[derive(PartialEq, Eq, Hash, Clone, ToBytes, FromBytes, Copy, Debug)]
pub enum TargetEnvironment {
    Building,
    Ground,
    Air,
    Naval,
    Projectile,
    Reclaim
}

#[derive(PartialEq, Eq, Hash, Clone, ToBytes, FromBytes, Copy)]
pub enum TargetTeam {
    Same,
    Different
}

#[derive(PartialEq, Clone, ToBytes, FromBytes)]
pub struct TargetPreFetcher {
    targets:HashMap<AllowedTarget, Option<Vec<usize>>>,
    keep_n:usize,
}

#[derive(PartialEq, Clone, ToBytes, FromBytes, Copy)]
pub struct AllowedTarget {
    env:TargetEnvironment,
    team:TargetTeam,
    range:f32,
}

pub enum AllowedTargets {
    SingleTarget(AllowedTarget),
    ManyTargets(Vec<AllowedTarget>),
    NoTargets
}

impl AllowedTarget {
    pub fn new(env:TargetEnvironment, team:TargetTeam, range:f32) -> Self {
        Self { env, team, range }
    }
    pub fn get_tid(&self, id:usize) -> TurboGameEngineTID {
        match self.env {
            TargetEnvironment::Air | TargetEnvironment::Ground | TargetEnvironment::Naval => TurboGameEngineTID::units(id),
            TargetEnvironment::Building => TurboGameEngineTID::buildings(id),
            TargetEnvironment::Projectile => TurboGameEngineTID::projectiles(id),
            TargetEnvironment::Reclaim => TurboGameEngineTID::reclaim(id),
        }
    }
}

impl Eq for AllowedTarget {

}

impl Hash for AllowedTarget {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.env.hash(state);
        self.team.hash(state);
        state.write_u32(self.range.to_bits());
    }
}

impl TargetPreFetcher {
    pub fn new(allowed_targets:Vec<AllowedTarget>, keep_n:usize) -> Self {
        let mut map = HashMap::with_capacity(allowed_targets.len());
        for key in allowed_targets {
            map.insert(key, None);
        }
        Self { targets: map, keep_n }
    }
    pub fn get_update_for_target<'a>(target:AllowedTarget, team:u16, position:Vec3Df, grid:&GridUsed, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) -> Vec<usize> {
        match target.env {
            TargetEnvironment::Building => match target.team {
                TargetTeam::Same => get_buildings_in_range_with_same_team(team, target.range, position, grid, buildings).collect(),
                TargetTeam::Different => get_buildings_in_range_with_diff_team(team, target.range, position, grid, buildings).collect(),
            },
            TargetEnvironment::Ground => match target.team {
                TargetTeam::Same => get_ground_units_in_range_with_same_team(team, target.range, position, grid, units).collect(),
                TargetTeam::Different => get_ground_units_in_range_with_diff_team(team, target.range, position, grid, units).collect(),
            },
            TargetEnvironment::Air => match target.team {
                TargetTeam::Same => get_air_units_in_range_with_same_team(team, target.range, position, grid, units).collect(),
                TargetTeam::Different => get_air_units_in_range_with_diff_team(team, target.range, position, grid, units).collect(),
            },
            TargetEnvironment::Naval => match target.team {
                TargetTeam::Same => get_naval_units_in_range_with_same_team(team, target.range, position, grid, units).collect(),
                TargetTeam::Different => get_naval_units_in_range_with_diff_team(team, target.range, position, grid, units).collect(),
            },
            TargetEnvironment::Projectile => get_projectiles_in_range_with_diff_team(team, target.range, position, grid, projectiles).collect(),
            TargetEnvironment::Reclaim => get_reclaim_in_range(target.range, position, grid, reclaim).collect(),
        }
    }
}

pub struct InTickPreFetcher {
    targets:HashMap<AllowedTarget, Vec<usize>>,
}

impl InTickPreFetcher {
    pub fn new() -> Self {
        Self { targets: HashMap::with_capacity(4) }
    }
    pub fn get_for_target<'a>(&mut self, target:AllowedTarget, team:u16, position:Vec3Df, grid:&GridUsed, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) -> &Vec<usize> {
        self.targets.entry(target.clone()).or_insert_with(|| {TargetPreFetcher::get_update_for_target(target, team, position, grid, units, buildings, projectiles, reclaim)})
    }
}

pub fn get_position_of_allowed_target<'a>(target:&AllowedTarget, id:usize, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) -> Vec3Df {
    match target.env {
        TargetEnvironment::Air | TargetEnvironment::Ground | TargetEnvironment::Naval => units.pos[id].pos,
        TargetEnvironment::Building => buildings.pos[id].pos,
        TargetEnvironment::Projectile => projectiles.pos[id].pos,
        TargetEnvironment::Reclaim => reclaim.pos[id].pos,
    }
}


pub fn get_position_of<'a>(target:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) -> Vec3Df {
    match target {
        TID::units(id) => units.pos[id].pos,
        TID::buildings(id) => buildings.pos[id].pos,
        TID::projectiles(id) => projectiles.pos[id].pos,
        TID::reclaim(id) => reclaim.pos[id].pos,
        _ => panic!("Can't get position of world")
    }
}


pub fn get_team_of<'a>(target:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) -> u16 {
    match target {
        TID::units(id) => units.stats[id].team,
        TID::buildings(id) => buildings.stats[id].team,
        TID::projectiles(id) => projectiles.stats[id].team,
        TID::reclaim(id) => reclaim.stats[id].team,
        _ => panic!("Can't get team of world")
    }
}

pub fn add_self_to_depend_set_of<'a>(self_id:TID, target_id:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) {
    match target_id {
        TID::units(id) => units.tunnels.depend_out.send(UnitEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), DependSetUpdate::Depend(self_id)))).unwrap(),
        TID::buildings(id) => buildings.tunnels.depend_out.send(BuildingEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), DependSetUpdate::Depend(self_id)))).unwrap(),
        TID::projectiles(id) => projectiles.tunnels.depend_out.send(ProjectileEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), DependSetUpdate::Depend(self_id)))).unwrap(),
        TID::reclaim(id) => reclaim.tunnels.depend_out.send(ReclaimEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), DependSetUpdate::Depend(self_id)))).unwrap(),
        _ => panic!("World can't depend on entities !")
    }
}

pub fn remove_self_from_depend_set_of<'a>(self_id:TID, target_id:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) {
    match target_id {
        TID::units(id) => units.tunnels.depend_out.send(UnitEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), DependSetUpdate::StopDepend(self_id)))).unwrap(),
        TID::buildings(id) => buildings.tunnels.depend_out.send(BuildingEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), DependSetUpdate::StopDepend(self_id)))).unwrap(),
        TID::projectiles(id) => projectiles.tunnels.depend_out.send(ProjectileEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), DependSetUpdate::StopDepend(self_id)))).unwrap(),
        TID::reclaim(id) => reclaim.tunnels.depend_out.send(ReclaimEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), DependSetUpdate::StopDepend(self_id)))).unwrap(),
        _ => panic!("World can't depend on entities !")
    }
}

pub fn heal_entity<'a>(heal_value:i32, self_id:TID, target_id:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>) {
    match target_id {
        TID::units(id) => units.tunnels.health_out.send(UnitEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), HealthUpdate::Heal(heal_value)))).unwrap(),
        TID::buildings(id) => buildings.tunnels.health_out.send(BuildingEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), HealthUpdate::Heal(heal_value)))).unwrap(),
        TID::projectiles(id) => projectiles.tunnels.health_out.send(ProjectileEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), SimpleHealthUpdate::Heal(heal_value)))).unwrap(),
        _ => panic!("World and reclaim can't be healed !")
    }
}



pub fn damage_entity<'a>(damage_value:i32, self_id:TID, target_id:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) {
    match target_id {
        TID::units(id) => units.tunnels.health_out.send(UnitEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), HealthUpdate::Damage(damage_value)))).unwrap(),
        TID::buildings(id) => buildings.tunnels.health_out.send(BuildingEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), HealthUpdate::Damage(damage_value)))).unwrap(),
        TID::projectiles(id) => projectiles.tunnels.health_out.send(ProjectileEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), SimpleHealthUpdate::Damage(damage_value)))).unwrap(),
        TID::reclaim(id) => reclaim.tunnels.health_out.send(ReclaimEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(self_id), SimpleHealthUpdate::Damage(damage_value)))).unwrap(),
        _ => panic!("World and reclaim can't be healed !")
    }
}

pub fn is_target_health_full<'a>(target_id:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) -> bool {
    match target_id {
        TID::units(id) => units.health[id].is_full(),
        TID::buildings(id) => buildings.health[id].is_full(),
        TID::projectiles(id) => projectiles.health[id].is_full(),
        TID::reclaim(id) => reclaim.health[id].is_full(),
        _ => panic!("World and reclaim can't be healed !")
    }
}
pub fn is_target_dead<'a>(target_id:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) -> bool {
    match target_id {
        TID::units(id) => units.health[id].is_dead(),
        TID::buildings(id) => buildings.health[id].is_dead(),
        TID::projectiles(id) => projectiles.health[id].is_dead(),
        TID::reclaim(id) => reclaim.health[id].is_dead(),
        _ => panic!("World and reclaim can't be healed !")
    }
}

pub fn entity_can_move(entity:TID) -> bool {
    match entity {
        TID::units(_) | TID::projectiles(_) => true,
        TID::buildings(_) | TID::reclaim(_) | TID::world | TID::teams(_) => false,
    }
}

pub fn do_death_accounting<'a>(dead:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>) {
    let dependents = match dead {
        TID::units(id) => &units.depend[id],
        TID::buildings(id) => &buildings.depend[id],
        TID::projectiles(id) => &projectiles.depend[id],
        TID::reclaim(id) => &projectiles.depend[id],
        TID::teams(_) | TID::world => panic!("Impossible !")     
    };
    for dependent in dependents.get_dependents().iter().map(|d| { *d}) {
        match dependent {
            TID::units(id) => {
                units.tunnels.depend_out.send(UnitEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(dead), DependSetUpdate::StopDepend(dead)))).unwrap();
                units.turrets[id].remove_dependency(dependent, dead, units, buildings);
            },
            TID::buildings(id) => {
                buildings.tunnels.depend_out.send(BuildingEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(dead), DependSetUpdate::StopDepend(dead)))).unwrap();
                buildings.turrets[id].remove_dependency(dependent, dead, units, buildings);
            },
            TID::projectiles(id) => projectiles.tunnels.depend_out.send(ProjectileEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(dead), DependSetUpdate::StopDepend(dead)))).unwrap(),
            TID::reclaim(id) => reclaim.tunnels.depend_out.send(ReclaimEvent::new(MustSync::Both, SimpleComponentEvent::new(id, Some(dead), DependSetUpdate::StopDepend(dead)))).unwrap(),
            _ => panic!("World can't depend on entities !")
        }
    }
}

pub fn get_resource_share_of<'a>(target:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>) -> i64 {
    match target {
        TID::units(id) => units.stats[id].resource_share,
        TID::buildings(id) => buildings.stats[id].resource_share,
        _ => panic!("Other types don't have resource shares")
    }
}

pub fn remove_resource_contribs_of<'a>(target:TID, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, teams:&TeamVecRead<'a, TurboGameEngineTID>) {
    match target {
        TID::units(id) => {
            let stats = &units.stats[id];
            let team = stats.team;
            let static_stats = &units.static_types[stats.get_id()];
            teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(target), TeamResourcesUpdate::RemoveStorage(static_stats.stats.resource_storage.clone()))));
            teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(target), TeamResourcesUpdate::RemoveProduction(static_stats.stats.resource_production.clone()))));
        },
        TID::buildings(id) => {
            let stats = &buildings.stats[id];
            let team = stats.team;
            let static_stats = &buildings.static_types[stats.get_id()];
            teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(target), TeamResourcesUpdate::RemoveStorage(static_stats.stats.resource_storage.clone()))));
            teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(target), TeamResourcesUpdate::RemoveProduction(static_stats.stats.resource_production.clone()))));
        },
        _ => panic!("Other types don't have resource shares")
    }
}

pub fn complex_stats_update_for<'a>(target:TID, source:Option<TID>, update:StatsUpdate, must_sync:MustSync, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>) {
    match target {
        TID::units(id) => {
            units.tunnels.stats_out.send(UnitEvent::new(must_sync, StatsEvent::new(id, source, update)));
        },
        TID::buildings(id) => {
            buildings.tunnels.stats_out.send(BuildingEvent::new(must_sync, StatsEvent::new(id, source, update)));
        },
        _ => panic!("Other types don't have complex stats")
    }
}

pub fn get_complex_stats_of<'a>(target:TID, units:&'a UnitVecRead<'a, TurboGameEngineTID>, buildings:&'a BuildingVecRead<'a, TurboGameEngineTID>) -> &'a Stats {
    match target {
        TID::units(id) => &units.stats[id],
        TID::buildings(id) => &buildings.stats[id],
        _ => panic!("Other types don't have complex stats")
    }
}

pub fn get_grid_index_for(env:TargetEnvironment) -> usize {
    match env {
        TargetEnvironment::Ground => GROUND_SET,
        TargetEnvironment::Projectile => PROJECTILE_SET,
        TargetEnvironment::Building => BUILDING_SET,
        TargetEnvironment::Air => AIR_SET,
        TargetEnvironment::Naval => NAVAL_SET,
        TargetEnvironment::Reclaim => RECLAIM_SET
    }
}

pub fn footprint_available<'a>(footprint:AABB, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, world:&GameMap<CoolVoxel, RTSGen>) -> bool {
    let z = footprint.get_first_point().z as i32;
    let zm1 = z - 1;
    let start_x = footprint.get_first_point().x as i32;
    let end_x = footprint.get_second_point().x as i32;
    let start_y = footprint.get_first_point().y as i32;
    let end_y = footprint.get_second_point().y as i32;
    let mut floor_good = true;
    'outer: for x in start_x..end_x {
        for y in start_y..end_y {
            let test = Vec3D::new(x, y, z);
            let test2 = Vec3D::new(x, y, zm1);
            if world.is_voxel_solid(test) || !world.is_voxel_solid(test2) {
                floor_good = false;
                break 'outer
            }
        }
    }
    if floor_good {
        let size = footprint.get_size();
        let center = footprint.get_center();
        let mut clear_of_buildings = true;
        for building in get_buildings_in_range(size, center, &world.set_grid, buildings) {
            if (*buildings.collider[building].get_global_aabb() + buildings.pos[building].pos).collision_aabb(&footprint) {
                clear_of_buildings = false;
                break;
            }
        }
        //println!("CLEAR : {clear_of_buildings}");
        clear_of_buildings
    }
    else {
        false
    }
}