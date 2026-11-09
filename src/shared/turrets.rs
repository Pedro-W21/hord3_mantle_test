use hord3::horde::{frontend::interact::Button::N, game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::{Identify, MustSync}, static_type_id::HasStaticTypeID, world::WorldComputeHandler}, geometry::{rotation::Rotation, vec3d::Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{entities::{building::{BuildingEvent, BuildingVecRead, NewBuilding}, projectile::{NewProjectile, ProjectileVecRead, projectile_stats::ProjectileStats}, reclaim::ReclaimVecRead, team::{TeamEvent, TeamVecRead}, unit::{NewUnit, UnitEvent, UnitVecRead}}, game_engine::CoolVoxel, game_map::{GameMap, GameMapEvent, GridUsed, rts_gen::RTSGen}, shared::{actions::{Action, AttackTarget, TID}, attack::{Attack, AttackCast, AttackType, PierceKind}, consts::{ASSIST_REPAIR_TARGETS, BUILDING_SET, TICKS_PER_SECOND, TICKS_PER_SECOND_F32, TURRET_ORIENT_TOLERANCE}, mesh_info::StaticMeshInfo, position::Position, resources::{ResourceCost, TeamResourceEvent, TeamResourcesUpdate}, simple_position::SimplePos, stats::{Stats, StatsEvent, StatsUpdate}, targeting::{AllowedTarget, AllowedTargets, InTickPreFetcher, TargetEnvironment, TargetTeam, add_self_to_depend_set_of, complex_stats_update_for, damage_entity, entity_can_move, footprint_available, get_complex_stats_of, get_position_of, get_position_of_allowed_target, get_resource_share_of, get_team_of, heal_entity, is_target_dead, is_target_health_full, remove_self_from_depend_set_of}}, turbo_game_engine::TurboGameEngineTID};

#[derive(Clone, Debug, PartialEq, FromBytes, ToBytes)]
pub struct Turrets {
    pub turrets:TurretsStore
}

impl Turrets {
    pub fn first_stage<'a>(&self, parent:TID, parent_pos:Vec3Df, static_type:&StaticTurrets, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>, teams:&TeamVecRead<'a, TID>) {
        let stats = get_complex_stats_of(parent, units, buildings);
        let resource_share = stats.compute_share(teams);
        if resource_share != stats.resource_share {
            complex_stats_update_for(parent, None, StatsUpdate::UpdateShare(resource_share), MustSync::Both, units, buildings);
        }
        match &self.turrets {
            TurretsStore::SingleTurret(turret) => {
                let static_turret_type = &static_type.turrets[turret.static_id];
                let new_ready = resource_share >= static_turret_type.resource_share_cutoff;
                if turret.ready != new_ready {
                    send_turret_event(parent, None, TurretsUpdate::UpdateSingleReady(new_ready), MustSync::Both, units, buildings);
                }
                
            },
            TurretsStore::MultiTurret(turrets) => {
                let mut readies = Vec::with_capacity(turrets.len());
                let mut any_diff = false;
                for turret in turrets {
                    let static_turret_type = &static_type.turrets[turret.static_id];
                    let new_ready = resource_share >= static_turret_type.resource_share_cutoff;
                    if new_ready != turret.ready {
                        any_diff = true;
                    }
                    readies.push(new_ready);
                }
                if any_diff {
                    send_turret_event(parent, None, TurretsUpdate::UpdateMultiReady(readies), MustSync::Both, units, buildings);
                }
            }
        }
    }

    pub fn second_stage<'a>(&self, targeting_data:&mut Option<InTickPreFetcher>, parent:TID, parent_pos:Vec3Df, parent_team:u16, static_type:&StaticTurrets, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>, grid: &GridUsed) {
        match &self.turrets {
            TurretsStore::SingleTurret(turret) => if turret.ready {
                let action = turret.get_action_to_perform(targeting_data, parent, parent_pos, parent_team, &static_type.turrets[0], units, buildings, reclaim, projectiles, grid);
                match action {
                    Some(act) => {
                        // println!("Turret of unit {:?} ready and is getting action {:?}", parent, act.clone());
                        send_turret_event(parent, None, TurretsUpdate::AddSingleAction(act), MustSync::Both, units, buildings);
                    },
                    None => ()
                }
                
            },
            TurretsStore::MultiTurret(turrets) => {
                let mut actions = Vec::with_capacity(turrets.len());
                let mut any_some = false;
                for turret in turrets {
                    if turret.ready {
                        let action = turret.get_action_to_perform(targeting_data, parent, parent_pos, parent_team, &static_type.turrets[turret.static_id], units, buildings, reclaim, projectiles, grid);
                        any_some = any_some || action.is_some();
                        actions.push(action);
                        
                    }
                }
                if any_some {
                    send_turret_event(parent, None, TurretsUpdate::AddMultiAction(actions), MustSync::Both, units, buildings)
                }
            },
        }
    }

    pub fn third_stage<'a>(&self, current_tick:usize, parent:TID, parent_pos:Vec3Df, static_type:&StaticTurrets, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>, teams:&TeamVecRead<'a, TID>, world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>,TID>) {
        match &self.turrets {
            TurretsStore::SingleTurret(turret) => {
                let action_update = match turret.perform_action(current_tick, parent, parent_pos, &static_type.turrets[turret.static_id], units, buildings, reclaim, projectiles, teams, world) {
                    PerformAction::CantTurnToTarget | PerformAction::PerformedAction { continue_action:false } => Some(PerformActionUpdate::RemoveAction),
                    PerformAction::TooClose | PerformAction::TooFarAway | PerformAction::NoAction | PerformAction::PerformedAction { continue_action:true } => None,
                    PerformAction::Progressing { new_data } => Some(PerformActionUpdate::Progressing { new_data }),
                    PerformAction::Turning { yaw_add, pitch_add } => Some(PerformActionUpdate::Turning { new_yaw: turret.yaw + yaw_add, new_pitch: turret.pitch + pitch_add })
                };
                match action_update {
                    Some(update) => send_turret_event(parent, None, TurretsUpdate::SinglePerformAction(update), MustSync::Both, units, buildings),
                    None => (),
                }
            }
            TurretsStore::MultiTurret(turrets) => {
                let mut updates = Vec::with_capacity(turrets.len());
                let mut any_some = false;
                for turret in turrets {
                    let action_update = match turret.perform_action(current_tick, parent, parent_pos, &static_type.turrets[turret.static_id], units, buildings, reclaim, projectiles, teams, world) {
                        PerformAction::CantTurnToTarget | PerformAction::PerformedAction { continue_action:false } => Some(PerformActionUpdate::RemoveAction),
                        PerformAction::TooClose | PerformAction::TooFarAway | PerformAction::NoAction | PerformAction::PerformedAction { continue_action:true } => None,
                        PerformAction::Progressing { new_data } => Some(PerformActionUpdate::Progressing { new_data }),
                        PerformAction::Turning { yaw_add, pitch_add } => Some(PerformActionUpdate::Turning { new_yaw: turret.yaw + yaw_add, new_pitch: turret.pitch + pitch_add })
                    };
                    any_some = any_some || action_update.is_some();
                    updates.push(action_update);
                }
                if any_some {
                    send_turret_event(parent, None, TurretsUpdate::MultiPerformAction(updates), MustSync::Both, units, buildings)
                }
            }
        }
    }
    pub fn fourth_stage<'a>(&self, parent:TID, team:u16, parent_pos:Vec3Df, static_type:&StaticTurrets, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>, teams:&TeamVecRead<'a, TID>) {
        // TODO : add to costs and don't update them, optimize to only send events when actually needed (costs change)
        match &self.turrets {
            TurretsStore::SingleTurret(turret) => {
                let static_turret_type = &static_type.turrets[turret.static_id];
                let added_cost = turret.get_acting_cost(parent, parent_pos, static_turret_type, units, buildings, reclaim, projectiles);
                let stats = get_complex_stats_of(parent, units, buildings);
                if added_cost != stats.resource_consumption {
                    if let Some(cost) = added_cost {
                        if let Some(old_cons) = stats.resource_consumption {
                            teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(parent), super::resources::TeamResourcesUpdate::RemoveConsumption(old_cons))));
                        }
                        teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(parent), super::resources::TeamResourcesUpdate::AddConsumption(cost))));
                    }
                    complex_stats_update_for(parent, None, StatsUpdate::UpdateConsumption(added_cost), MustSync::Both, units, buildings);
                }
            },
            TurretsStore::MultiTurret(turrets) => {
                let mut total_cost = None;
                for turret in turrets {
                    let static_turret_type = &static_type.turrets[turret.static_id];
                    let added_cost = turret.get_acting_cost(parent, parent_pos, static_turret_type, units, buildings, reclaim, projectiles);
                    match total_cost.as_mut() {
                        Some(cost) => *cost = *cost + added_cost.unwrap_or(ResourceCost::new(0, 0, 0)),
                        None => total_cost = added_cost
                    }
                }
                let stats = get_complex_stats_of(parent, units, buildings);
                if total_cost != stats.resource_consumption {

                    if let Some(cost) = total_cost {
                        if let Some(old_cons) = stats.resource_consumption {
                            teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(parent), super::resources::TeamResourcesUpdate::RemoveConsumption(old_cons))));
                        }
                        teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(parent), super::resources::TeamResourcesUpdate::AddConsumption(cost))));
                    }
                    complex_stats_update_for(parent, None, StatsUpdate::UpdateConsumption(total_cost), MustSync::Both, units, buildings);
                }
            }
        }
        
        // TODO : add events generated
    }

    pub fn apply_turrets_update(&mut self, update:TurretsUpdate) {
        match update {
            TurretsUpdate::UpdateSingleReady(ready) => self.turrets.unwrap_single().ready = ready,
            TurretsUpdate::UpdateMultiReady(readies) => self.turrets.unwrap_multi().iter_mut().zip(readies).for_each(|(t, r)| {t.ready = r}),
            TurretsUpdate::AddSingleAction(act) => self.turrets.unwrap_single().action_slot = Some(act),
            TurretsUpdate::AddMultiAction(acts) => self.turrets.unwrap_multi().iter_mut().zip(acts).for_each(|(t, act)| {
                match act {
                    Some(act) => t.action_slot = Some(act),
                    None => ()
                }
            }),
            TurretsUpdate::SinglePerformAction(perf) => match perf {
                PerformActionUpdate::Turning { new_yaw, new_pitch } => {
                    let turret = self.turrets.unwrap_single();
                    turret.yaw = new_yaw;
                    turret.pitch = new_pitch;
                },
                PerformActionUpdate::RemoveAction => self.turrets.unwrap_single().action_slot = None,
                PerformActionUpdate::Progressing { new_data } => self.turrets.unwrap_single().action_slot.as_mut().unwrap().kind = new_data
            },
            TurretsUpdate::MultiPerformAction(perfs) => self.turrets.unwrap_multi().iter_mut().zip(perfs).for_each(|(turret, perf)| {
                if let Some(perf) = perf {
                    match perf {
                        PerformActionUpdate::Turning { new_yaw, new_pitch } => {
                            turret.yaw = new_yaw;
                            turret.pitch = new_pitch;
                        },
                        PerformActionUpdate::RemoveAction => turret.action_slot = None,
                        PerformActionUpdate::Progressing { new_data } => turret.action_slot.as_mut().unwrap().kind = new_data
                    }
                }
            }),
            TurretsUpdate::SingleWipeAction => self.turrets.unwrap_single().action_slot = None,
            TurretsUpdate::MultiWipeAction(wipes) => self.turrets.unwrap_multi().iter_mut().zip(wipes).for_each(|(turret, wipe)| {
                if wipe {
                    turret.action_slot = None
                }
            }),
        }
    }
    pub fn remove_dependency<'a>(&self, parent:TID, dependency:TID, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>) {
        match &self.turrets {
            TurretsStore::SingleTurret(turret) => if turret.should_remove_dependency(parent, dependency, units, buildings) {
                send_turret_event(parent, Some(dependency), TurretsUpdate::SingleWipeAction, MustSync::Both, units, buildings)
            },
            TurretsStore::MultiTurret(turrets) => {
                let mut wipes = Vec::with_capacity(turrets.len());
                for turret in turrets {
                    wipes.push(turret.should_remove_dependency(parent, dependency, units, buildings));
                }
                send_turret_event(parent, Some(dependency), TurretsUpdate::MultiWipeAction(wipes), MustSync::Both, units, buildings)
                
            }
        }
    }
}

pub fn send_turret_event<'a>(id:TID, source:Option<TID>, update:TurretsUpdate, must_sync:MustSync, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>) {
    match id {
        TID::units(id) => units.tunnels.turrets_out.send(UnitEvent::new(must_sync, TurretsEvent::<TID>::new(id, source, update))).unwrap(),
        TID::buildings(id) => buildings.tunnels.turrets_out.send(BuildingEvent::new(must_sync, TurretsEvent::<TID>::new(id, source, update))).unwrap(),
        _ => panic!("This doesn't have a turret !")
    }
}

#[derive(Clone, Debug, PartialEq, FromBytes, ToBytes)]
pub enum TurretsStore {
    SingleTurret(Turret),
    MultiTurret(Vec<Turret>)
}

impl TurretsStore {
    pub fn unwrap_single(&mut self) -> &mut Turret {
        match self {
            Self::SingleTurret(turret) => turret,
            _ => panic!("Not a SingleTurret !")
        }
    }
    pub fn unwrap_single_ref(&self) -> &Turret {
        match self {
            Self::SingleTurret(turret) => turret,
            _ => panic!("Not a SingleTurret !")
        }
    }
    pub fn unwrap_multi(&mut self) -> &mut Vec<Turret> {
        match self {
            Self::MultiTurret(turrets) => turrets,
            _ => panic!("Not a MultiTurret !")
        }
    }
    pub fn unwrap_multi_ref(&self) -> &Vec<Turret> {
        match self {
            Self::MultiTurret(turrets) => turrets,
            _ => panic!("Not a MultiTurret !")
        }
    }
}

#[derive(Clone, Debug, PartialEq, FromBytes, ToBytes)]
pub enum TurretsUpdate {
    UpdateSingleReady(bool),
    UpdateMultiReady(Vec<bool>),
    AddSingleAction(TurretAction),
    AddMultiAction(Vec<Option<TurretAction>>),
    SinglePerformAction(PerformActionUpdate),
    MultiPerformAction(Vec<Option<PerformActionUpdate>>),
    SingleWipeAction,
    MultiWipeAction(Vec<bool>),
}

#[derive(Clone, Debug, PartialEq, FromBytes, ToBytes)]
pub enum PerformActionUpdate {
    Turning {new_yaw:f32, new_pitch:f32},
    Progressing {new_data:TurretActionKind},
    RemoveAction
}

#[derive(Clone, Debug, PartialEq, FromBytes, ToBytes)]
pub struct Turret {
    pub static_id:usize,
    pub yaw:f32,
    pub pitch:f32,
    current_health:f32,
    pub action_slot:Option<TurretAction>,
    ready:bool,
    pub static_type_pos:u8,
}

pub enum PerformAction {
    TooFarAway,
    TooClose,
    CantTurnToTarget,
    Turning{yaw_add:f32, pitch_add:f32},
    Progressing {new_data:TurretActionKind},
    PerformedAction {continue_action:bool},
    NoAction,
}

pub type TurretsEvent<ID:Identify> = SimpleComponentEvent<ID, TurretsUpdate>;

impl<ID:Identify> SimpleComponentUpdate<Turrets, ID> for TurretsUpdate {
    fn apply_to_comp(self, component:&mut Turrets) {
        component.apply_turrets_update(self);
    }
}

impl StaticComponent for StaticTurrets {

}

impl<ID:Identify> Component<ID> for Turrets {
    type CE = TurretsEvent<ID>;
    type SC = StaticTurrets;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self {
            turrets:if static_comp.turrets.len() == 1 {
                if static_comp.turrets[0].positions.len() == 1 {
                    TurretsStore::SingleTurret(Turret::from_static(&static_comp.turrets[0], 0, 0))
                }
                else {
                    let mut turrets = Vec::with_capacity(4);
                    for i in 0..static_comp.turrets.len() {
                        let static_type = &static_comp.turrets[i];
                        for pos in 0..static_type.positions.len() {
                            turrets.push(Turret::from_static(static_type, i, pos as u8));
                        }
                    }
                    TurretsStore::MultiTurret(turrets)
                }
            }
            else {
                let mut turrets = Vec::with_capacity(4);
                for i in 0..static_comp.turrets.len() {
                    let static_type = &static_comp.turrets[i];
                    for pos in 0..static_type.positions.len() {
                        turrets.push(Turret::from_static(static_type, i, pos as u8));
                    }
                }
                TurretsStore::MultiTurret(turrets)
            } 
        }
    }
}

impl Turret {
    pub fn from_static(static_type:&StaticTurret, static_id:usize, static_type_pos:u8) -> Self {
        Self { static_id, yaw:0.0, pitch: 0.0, current_health: static_type.max_health, action_slot: None, ready: false, static_type_pos }
    }
    /// Returns if action slot needs to be wiped
    pub fn should_remove_dependency<'a>(&self, parent:TID, dependency:TID, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>) -> bool {
        match &self.action_slot {
            Some(act) => match &act.kind {
                TurretActionKind::Attack { cast } => match &cast.target {
                    AttackTarget::Entity(ent) => *ent == dependency,
                    AttackTarget::Position(_) => false
                },
                TurretActionKind::Assist { target, hold } => *target == dependency,
                TurretActionKind::Repair { target, hold } => *target == dependency,
                TurretActionKind::Reclaim { target } => *target == dependency,
                TurretActionKind::BuildBuilding { building_type, building, building_space } => match building {
                    Some(id) => TID::buildings(*id) == dependency,
                    None => false
                },
                TurretActionKind::BuildUnit { unit_type, progress } => false
            },
            None => false
        }
    }
    /// Used in first stage
    pub fn get_acting_cost<'a>(&self, parent:TID, parent_pos:Vec3Df, static_type:&StaticTurret, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>) -> Option<ResourceCost> {
        match &self.action_slot {
            Some(turret_action) => 
            {
                // TODO potential optimization : add fast path for an attack with no attack cost (which is going to be most attacks and don't need to check everything else)
                let self_pos = parent_pos;

                let target_pos = match turret_action.kind.clone() {
                        TurretActionKind::Attack { cast } => match cast.target {
                        AttackTarget::Position(pos) => pos,
                        AttackTarget::Entity(ent) => get_position_of(ent, units, buildings, projectiles, reclaim),
                    },
                    TurretActionKind::Assist { target, hold } => get_position_of(target, units, buildings, projectiles, reclaim),
                    TurretActionKind::Reclaim { target } => get_position_of(target, units, buildings, projectiles, reclaim),
                    TurretActionKind::Repair { target, hold } => get_position_of(target, units, buildings, projectiles, reclaim),
                    TurretActionKind::BuildUnit { unit_type, progress } => parent_pos,
                    TurretActionKind::BuildBuilding { building_type, building, building_space } => match building {
                        Some(building_id) => get_position_of(TID::buildings(building_id), units, buildings, projectiles, reclaim),
                        None => building_space
                    }
                };

                let range = static_type.allowed_actions.get_range_at(turret_action.allowed_id);
                let self_target_dist = self_pos.dist(&target_pos);
                if self_target_dist > range.0 && self_target_dist < range.1 {
                    match &turret_action.kind {
                        TurretActionKind::Attack { cast } => {
                            let attack = static_type.allowed_actions.get_attack();
                            if cast.current_charge == 0 {
                                attack.cost.clone()
                            }
                            else {
                                attack.cost_during_charge.clone()
                            }
                        },
                        TurretActionKind::Assist { target, hold } => {
                            match target {
                                TurboGameEngineTID::units(unit) => Some(units.static_types[units.stats[*unit].get_id()].stats.build_contribution_cost),
                                TurboGameEngineTID::buildings(building) => Some(buildings.static_types[buildings.stats[*building].get_id()].stats.build_contribution_cost),
                                _ => None
                            }
                        },
                        TurretActionKind::Reclaim { target } => {
                            None
                        },
                        TurretActionKind::Repair { target, hold } => {
                            match target {
                                TurboGameEngineTID::units(unit) => Some(units.static_types[units.stats[*unit].get_id()].stats.build_contribution_cost),
                                TurboGameEngineTID::buildings(building) => Some(buildings.static_types[buildings.stats[*building].get_id()].stats.build_contribution_cost),
                                _ => None
                            }
                        },
                        TurretActionKind::BuildUnit { unit_type, progress } => {
                            Some(units.static_types[*unit_type].stats.build_contribution_cost)
                        },
                        TurretActionKind::BuildBuilding { building_type, building, building_space } => {
                            Some(buildings.static_types[*building_type].stats.build_contribution_cost)
                        }
                    }
                }
                else {
                    None
                }

                
            },
            None => None
        }
    }
    /// autonomously gets a new action when doesn't have any, otherwise returns None (don't need to update current action/no action to update with)
    pub fn get_action_to_perform<'a>(&self, targeting_data:&mut Option<InTickPreFetcher>, parent:TID, parent_pos:Vec3Df, parent_team:u16, static_type:&StaticTurret, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>, grid: &GridUsed) -> Option<TurretAction> {
        match &self.action_slot {
            Some(action) => None,
            None => match &static_type.allowed_actions {
                AllowedActions::SingleAction(action) => self.get_action_for_allowed_targets(action, 0, targeting_data, parent, parent_pos, parent_team, static_type, units, buildings, reclaim, projectiles, grid),
                AllowedActions::ManyActions(actions) => {
                    let mut action_id = 0;
                    for action in actions {
                        let turret_action = self.get_action_for_allowed_targets(action, action_id, targeting_data, parent, parent_pos, parent_team, static_type, units, buildings, reclaim, projectiles, grid);
                        if turret_action.is_some() {
                            return turret_action
                        }
                        action_id += 1;
                    }
                    None
                },
            },
        }
    }
    fn get_action_for_allowed_targets<'a>(&self, action:&AllowedAction, allowed_id:usize, targeting_data:&mut Option<InTickPreFetcher>, parent:TID, parent_pos:Vec3Df, parent_team:u16, static_type:&StaticTurret, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>, grid: &GridUsed) -> Option<TurretAction> {
        match action.get_targets() {
            AllowedTargets::NoTargets => None,
            AllowedTargets::SingleTarget(target) => self.get_action_for_target(action, target, allowed_id, targeting_data, parent, parent_pos, parent_team, static_type, units, buildings, reclaim, projectiles, grid),
            AllowedTargets::ManyTargets(targets) => {
                for (i, target) in targets.into_iter().enumerate() {
                    let turret_action = self.get_action_for_target(action, target, allowed_id, targeting_data, parent, parent_pos, parent_team, static_type, units, buildings, reclaim, projectiles, grid);
                    if turret_action.is_some() {
                        return turret_action
                    }
                }
                None
            }
        }
    }
    fn get_action_for_target<'a>(&self, action:&AllowedAction, target:AllowedTarget, allowed_id:usize, targeting_data:&mut Option<InTickPreFetcher>, parent:TID, parent_pos:Vec3Df, parent_team:u16, static_type:&StaticTurret, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>, grid: &GridUsed) -> Option<TurretAction> {
        let potential_target_ids = 
        match targeting_data {
            Some(in_tick) => in_tick.get_for_target(target, parent_team, parent_pos, grid, units, buildings, projectiles, reclaim),
            None => {
                let mut in_tick = InTickPreFetcher::new();
                *targeting_data = Some(in_tick);
                let target_ids = targeting_data.as_mut().unwrap().get_for_target(target, parent_team, parent_pos, grid, units, buildings, projectiles, reclaim);
                target_ids
            }
        };
        let mut closest = None;
        let mut closest_dist = f32::INFINITY;
        for id in potential_target_ids {
            let target_pos = get_position_of_allowed_target(&target, *id, units, buildings, projectiles, reclaim);
            let dist = target_pos.dist(&parent_pos);
            if dist < closest_dist {
                closest = Some(*id);
                closest_dist = dist;
            }
        }
        if potential_target_ids.len() > 0 {
            // dbg!(action, closest, potential_target_ids.len());
        }
        match closest {
            Some(closest_id) => {
                let target_id = target.get_tid(closest_id);
                add_self_to_depend_set_of(parent, target_id, units, buildings, projectiles, reclaim);
                Some(
                    TurretAction {
                        kind: match action {
                            AllowedAction::Attack(atk) => TurretActionKind::Attack { cast: AttackCast { current_charge: atk.charge_ticks.unwrap_or(0), target: AttackTarget::Entity(target_id) } },
                            AllowedAction::Assist { assist_rate, resource_multiplier, range } => TurretActionKind::Assist { target: target_id, hold: false },
                            AllowedAction::Repair { repair_rate, resource_multiplier, range } => TurretActionKind::Repair { target: target_id, hold: false },
                            AllowedAction::Reclaim { reclaim_rate, resource_multiplier, range } => TurretActionKind::Reclaim { target: target_id },
                            _ => panic!("Impossible to automatically build a unit or building")
                        },
                        allowed_id
                    }
                )
            },
            None => None
        }
    }

    pub fn get_launch_params<'a>(&self, projectile_static_id:usize, target:Vec3Df, target_id:Option<TurboGameEngineTID>, current_tick:usize, parent:TID, parent_pos:Vec3Df, self_pos:Vec3Df, static_type:&StaticTurret, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>) -> (Vec3Df, f32) {
        let static_projectile = &projectiles.static_types[projectile_static_id];
        /// what the fuck
        /// how to compute necessary initial speed to hit the intended target
        /// What to aim for ?
        ///     - current enemy position
        ///     - enemy position in (position + speed * travel_time) ?
        ///     - expected enemy position in (travel_time) seconds ?
        /// How to compute :
        ///     - pos_at = initial_speed * time + (1/2) * g * time^2
        ///     - g can be 3D
        ///     - need to choose time otherwise 5 unknowns
        ///     - choosing time = hit time also eliminates choosing pos_at, which becomes final position
        // TODO : choose better target
        // pre-adjust to center on shooter rather than world center
        let pos_diff = target - self_pos;
        let travel_time = (pos_diff.norme()/static_projectile.stats.max_speed) * TICKS_PER_SECOND_F32;
        let initial_speed = (pos_diff/travel_time - static_projectile.stats.gravity_felt * travel_time * 0.5);
        //dbg!(travel_time, initial_speed);
        (initial_speed, travel_time)
    }
    pub fn launch_projectile<'a>(&self, initial_speed:Vec3Df, travel_time:f32, projectile_static_id:usize, target:Vec3Df, target_id:Option<TurboGameEngineTID>, current_tick:usize, parent:TID, parent_pos:Vec3Df, self_pos:Vec3Df, static_type:&StaticTurret, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>) {
        let proj_id = projectiles.get_next_ent_id();
        let new_proj = NewProjectile::new(
            Position::new().with_pos(self_pos).with_speed(initial_speed),
            ProjectileStats::new(get_team_of(parent, units, buildings, projectiles, reclaim), travel_time as usize + TICKS_PER_SECOND, current_tick, projectile_static_id),
            MustSync::Both,
            Some(parent),
            Some(proj_id)
        );
        // println!("Creating new projectile {}", proj_id.get_id());
        projectiles.tunnels.new_ents.send(new_proj).unwrap();
    }
    /// Used in penultimate stage ?
    pub fn perform_action<'a>(&self, current_tick:usize, parent:TID, parent_pos:Vec3Df, static_type:&StaticTurret, units:&UnitVecRead<'a, TID>, buildings:&BuildingVecRead<'a, TID>, reclaim:&ReclaimVecRead<'a, TID>, projectiles:&ProjectileVecRead<'a, TID>, teams:&TeamVecRead<'a, TID>, world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TID>) -> PerformAction {
        match &self.action_slot {
            Some(turret_action) => {
                let self_pos = match parent {
                    TID::units(unit_id) => {
                        parent_pos + Rotation::from_orientation(units.pos[unit_id].orientation).rotate(static_type.positions[self.static_type_pos as usize])
                    },
                    TID::buildings(_) => {
                        parent_pos + static_type.positions[self.static_type_pos as usize]
                    },
                    _ => panic!("Other entities don't have turrets")
                };

                'must_turn: {
                    if static_type.must_turn {
                        let target_pos = match turret_action.kind.clone() {
                            TurretActionKind::Assist { target, .. } | TurretActionKind::Reclaim { target } | TurretActionKind::Repair { target, .. } => get_position_of(target, units, buildings, projectiles, reclaim),
                            TurretActionKind::BuildBuilding { building_type, building, building_space } => match building {
                                Some(building_id) => get_position_of(TID::buildings(building_id), units, buildings, projectiles, reclaim),
                                None => building_space
                            },
                            _ => break 'must_turn
                        };
                        let direct_orient = self_pos.get_orient_vers(&target_pos);
                        let yaw_diff = direct_orient.0 - self.yaw;
                        let pitch_diff = direct_orient.1 - self.pitch;
                        if yaw_diff.abs() > TURRET_ORIENT_TOLERANCE || pitch_diff.abs() > TURRET_ORIENT_TOLERANCE {
                            //println!("UNIT {:?} TURNING {} {}", parent, yaw_diff, pitch_diff);
                            return PerformAction::Turning { yaw_add: yaw_diff.clamp(-static_type.max_turn_rate, static_type.max_turn_rate), pitch_add: pitch_diff.clamp(-static_type.max_elevation_rate, static_type.max_elevation_rate) }
                        }
                    }
                }
                match turret_action.kind.clone() {
                    TurretActionKind::Attack { cast } => {
                        if cast.current_charge == 0 {
                            match cast.target {
                                AttackTarget::Entity(target_id) => {
                                    if is_target_dead(target_id, units, buildings, projectiles, reclaim) {
                                        //panic!("Got into impossible state");
                                        // This should be impossible to enter
                                        PerformAction::PerformedAction { continue_action: false }
                                    }
                                    else {
                                        let attack = static_type.allowed_actions.get_attack();
                                        let target_position = get_position_of(target_id, units, buildings, projectiles, reclaim);
                                        let distance_to_target = parent_pos.dist(&target_position);
                                        if distance_to_target > attack.range.0 && distance_to_target < attack.range.1 {
                                            match &attack.attack_type {
                                                AttackType::Hitscan { damage, pierce, beam_width, end_of_beam, continuous } => {
                                                    if static_type.must_turn {
                                                        let direct_orient = self_pos.get_orient_vers(&target_position);
                                                        let yaw_diff = direct_orient.0 - self.yaw;
                                                        let pitch_diff = direct_orient.1 - self.pitch;
                                                        if yaw_diff.abs() > TURRET_ORIENT_TOLERANCE || pitch_diff.abs() > TURRET_ORIENT_TOLERANCE {
                                                            return PerformAction::Turning { yaw_add: yaw_diff.clamp(-static_type.max_turn_rate, static_type.max_turn_rate), pitch_add: pitch_diff.clamp(-static_type.max_elevation_rate, static_type.max_elevation_rate) }
                                                        }
                                                    }
                                                    match pierce {
                                                        PierceKind::AlwaysHitTarget => {
                                                            // TODO : support complex damage with effects
                                                            damage_entity(damage.get_simple_damage(), parent, target_id, units, buildings, projectiles, reclaim);
                                                        },
                                                        _ => todo!("Not supported yet")
                                                    }
                                                },
                                                AttackType::Projectile { id } => {
                                                    
                                                    let target = target_position;
                                                    let (init_speed, travel_time) = self.get_launch_params(*id, target, Some(target_id), current_tick, parent, parent_pos, self_pos, static_type, units, buildings, reclaim, projectiles);
                                                    let direct_orient = Vec3Df::zero().get_orient_vers(&init_speed);
                                                    let yaw_diff = direct_orient.0 - self.yaw;
                                                    let pitch_diff = direct_orient.1 - self.pitch;
                                                    if yaw_diff.abs() > TURRET_ORIENT_TOLERANCE || pitch_diff.abs() > TURRET_ORIENT_TOLERANCE {
                                                        return PerformAction::Turning { yaw_add: yaw_diff.clamp(-static_type.max_turn_rate, static_type.max_turn_rate), pitch_add: pitch_diff.clamp(-static_type.max_elevation_rate, static_type.max_elevation_rate) }
                                                    }
                                                    self.launch_projectile(init_speed, travel_time, *id, target, Some(target_id), current_tick, parent, parent_pos, self_pos, static_type, units, buildings, reclaim, projectiles);
                                                },
                                                AttackType::ManyProjectiles { ids } => {
                                                    let target = target_position;
                                                    for id in ids {
                                                        let (init_speed, travel_time) = self.get_launch_params(*id, target, Some(target_id), current_tick, parent, parent_pos, self_pos, static_type, units, buildings, reclaim, projectiles);
                                                        self.launch_projectile(init_speed, travel_time, *id, target, Some(target_id), current_tick, parent, parent_pos, self_pos, static_type, units, buildings, reclaim, projectiles);
                                                
                                                    }
                                                }
                                            }
                                            PerformAction::Progressing { new_data: TurretActionKind::Attack { cast:AttackCast { current_charge: attack.charge_ticks.unwrap_or(0), target: cast.target.clone() } } }
                                        }
                                        else if distance_to_target >= attack.range.1 {
                                            if entity_can_move(parent) {
                                                PerformAction::TooFarAway

                                            }
                                            else {
                                                remove_self_from_depend_set_of(parent, target_id, units, buildings, projectiles, reclaim);
                                                PerformAction::PerformedAction { continue_action: false }
                                            }
                                        }
                                        else {
                                            if entity_can_move(parent) {
                                                PerformAction::TooClose
                                            }
                                            else {
                                                remove_self_from_depend_set_of(parent, target_id, units, buildings, projectiles, reclaim);
                                                PerformAction::PerformedAction { continue_action: false }
                                            }
                                        }
                                    }
                                },
                                AttackTarget::Position(pos) => {
                                    let attack = static_type.allowed_actions.get_attack();
                                    PerformAction::Progressing { new_data: TurretActionKind::Attack { cast:AttackCast { current_charge: attack.charge_ticks.unwrap_or(0), target: cast.target.clone() } } }
                                },
                            }
                        }
                        else {
                            let new_kind = TurretActionKind::Attack { cast:AttackCast { current_charge: cast.current_charge - 1, target: cast.target.clone() } };
                            PerformAction::Progressing { new_data: new_kind }
                        }
                    },
                    TurretActionKind::Repair { target, hold } => {
                        if is_target_health_full(target, units, buildings, projectiles, reclaim) {
                            if !hold {
                                remove_self_from_depend_set_of(parent, target, units, buildings, projectiles, reclaim);
                            }
                            PerformAction::PerformedAction { continue_action: hold }
                        }
                        else {
                            let (repair_rate, _, range) = static_type.allowed_actions.get_repair_at(turret_action.allowed_id);
                            if parent_pos.dist(&get_position_of(target, units, buildings, projectiles, reclaim)) < range {
                                heal_entity(repair_rate, parent, target, units, buildings, projectiles);
                                PerformAction::Progressing { new_data: TurretActionKind::Repair { target, hold } }
                            }
                            else if hold && entity_can_move(parent) {
                                PerformAction::TooFarAway
                            }
                            else {
                                remove_self_from_depend_set_of(parent, target, units, buildings, projectiles, reclaim);
                                PerformAction::PerformedAction { continue_action: false }
                            }
                        }
                    },
                    TurretActionKind::Reclaim { target } => {
                        if is_target_dead(target, units, buildings, projectiles, reclaim) {
                            // Should be impossible to reach !!!!
                            PerformAction::PerformedAction { continue_action: false }
                        }
                        else {
                            let (reclaim_rate, _, range) = static_type.allowed_actions.get_reclaim_at(turret_action.allowed_id);
                            if parent_pos.dist(&get_position_of(target, units, buildings, projectiles, reclaim)) < range {
                                damage_entity(reclaim_rate, parent, target, units, buildings, projectiles, reclaim);
                                PerformAction::Progressing { new_data: TurretActionKind::Reclaim { target } }
                            }
                            else {
                                remove_self_from_depend_set_of(parent, target, units, buildings, projectiles, reclaim);
                                PerformAction::PerformedAction { continue_action: false }
                            }
                        }
                    },
                    TurretActionKind::BuildUnit { unit_type, progress } => {
                        let unit_static = &units.static_types[unit_type];
                        if progress >= unit_static.stats.construction_quantity {
                            let pos = Position::new().with_pos(parent_pos + static_type.allowed_actions.get_placement_at(turret_action.allowed_id));
                            let team = get_team_of(parent, units, buildings, projectiles, reclaim);
                            teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(parent), TeamResourcesUpdate::AddProdAndStorage(unit_static.stats.resource_production, unit_static.stats.resource_storage))));
                            let stats = Stats::new(team, unit_type, unit_static.stats.default_env, 0);
                            units.tunnels.new_ents.send(NewUnit::new(pos, stats, MustSync::Both, Some(parent), None));
                            PerformAction::PerformedAction { continue_action: false }
                        }
                        else {
                            PerformAction::Progressing { new_data: TurretActionKind::BuildUnit { unit_type, progress:progress + static_type.allowed_actions.get_build_rate_at(turret_action.allowed_id) * get_resource_share_of(parent, units, buildings) as i32 } }
                        }
                    },
                    TurretActionKind::BuildBuilding { building_type, building, building_space } => {
                        if parent_pos.dist(&building_space) < static_type.allowed_actions.get_range_at(turret_action.allowed_id).1 {
                            match building {
                                Some(build) => if buildings.stats[build].upgrade_progress > 0 {
                                    if !buildings.depend[build].get_dependents().contains(&parent) {
                                        add_self_to_depend_set_of(parent, TID::buildings(build), units, buildings, projectiles, reclaim);
                                    }
                                    buildings.tunnels.stats_out.send(BuildingEvent::new(MustSync::Both, StatsEvent::new(build, Some(parent), StatsUpdate::AddToUpgradeProgress(-static_type.allowed_actions.get_build_rate_at(turret_action.allowed_id) * get_resource_share_of(parent, units, buildings) as i32))));
                                    PerformAction::Progressing { new_data: turret_action.kind.clone() }
                                }
                                else {
                                    PerformAction::PerformedAction { continue_action: false }
                                },
                                None => {
                                    let static_building = &buildings.static_types[building_type];
                                    if footprint_available(static_building.stats.building_footprint + building_space, buildings, &world.world) {
                                        let new_building = buildings.get_next_ent_id();
                                        //println!("FOOTPRINT AVAILABLE FOR BUILDING {} AS ENGINEER {:?} !", new_building.get_id(), parent);
                                        let pos = SimplePos {pos:building_space};
                                        let team = get_team_of(parent, units, buildings, projectiles, reclaim);
                                        teams.tunnels.resources_out.send(TeamEvent::new(MustSync::Both, TeamResourceEvent::new(team as usize, Some(parent), TeamResourcesUpdate::AddProdAndStorage(static_building.stats.resource_production, static_building.stats.resource_storage))));
                                        let stats = Stats::new(team, building_type, static_building.stats.default_env, static_building.stats.construction_quantity);
                                        buildings.tunnels.new_ents.send(NewBuilding::new(pos, stats, MustSync::Both, Some(parent), Some(new_building.clone())));
                                        if let Some(update) = world.world.set_grid.get_point_move_update(&(Vec3Df::all_ones() * 100_000.0), &building_space, new_building.get_id(), BUILDING_SET) {
                                            world.tunnels.send_event(GameMapEvent::UpdateSetGrid(update));
                                        }
                                        
                                        PerformAction::Progressing { new_data: TurretActionKind::BuildBuilding { building_type, building:Some(new_building.get_id()), building_space } }
                                    }
                                    else {
                                        PerformAction::PerformedAction { continue_action: false }
                                    }
                                }
                            }
                        }
                        else {
                            PerformAction::TooFarAway
                        }
                    },
                    _ => panic!("Not implemented yet")
                }
            },
            None => PerformAction::NoAction
        }
    }
}

#[derive(Clone)]
pub struct StaticTurrets {
    pub turrets:Vec<StaticTurret>
}

impl StaticTurrets {
    pub fn get_full_count(&self) -> usize {
        let mut total = 0;
        for turret in &self.turrets {
            total += turret.positions.len();
        }
        total
    }
}

#[derive(Clone, Debug)]
pub enum AllowedAction {
    Attack(Attack),
    Assist{assist_rate:f32, resource_multiplier:f32, range:f32},
    Repair{repair_rate:i32, resource_multiplier:f32, range:f32},
    Reclaim{reclaim_rate:i32, resource_multiplier:f32, range:f32},
    BuildUnit{units:BuildableUnits, build_rate:i32, range:f32, placement:Vec3Df},
    BuildBuilding{buildings:BuildableBuildings, build_rate:i32, range:f32, placement:Vec3Df}
}

impl AllowedAction {
    pub fn get_attack(&self) -> &Attack {
        match &self {
            AllowedAction::Attack(a) => a,
            _ => panic!("Not an attack")
        }
    }
    pub fn get_targets(&self) -> AllowedTargets {
        match self {
            Self::Attack(atk) => AllowedTargets::ManyTargets(atk.target_envs.iter().map(|env| {AllowedTarget::new(env.clone(), TargetTeam::Different, atk.range.1)}).collect()),
            Self::Assist { assist_rate, resource_multiplier, range } => AllowedTargets::ManyTargets(ASSIST_REPAIR_TARGETS.map(|(env,team)| {AllowedTarget::new(env, team, *range)}).into_iter().collect()), 
            Self::Repair { repair_rate, resource_multiplier, range } => AllowedTargets::ManyTargets(ASSIST_REPAIR_TARGETS.map(|(env,team)| {AllowedTarget::new(env, team, *range)}).into_iter().collect()),
            Self::Reclaim { reclaim_rate, resource_multiplier, range } => AllowedTargets::SingleTarget(AllowedTarget::new(TargetEnvironment::Reclaim, TargetTeam::Different, *range)),
            Self::BuildBuilding { buildings, build_rate, range, placement } => AllowedTargets::NoTargets,
            Self::BuildUnit { units, build_rate, range, placement } => AllowedTargets::NoTargets
        }
    }
    pub fn get_repair(&self) -> (i32, f32, f32) {
        match &self {
            AllowedAction::Repair { repair_rate, resource_multiplier, range } => (*repair_rate, *resource_multiplier, *range),
            _ => panic!("Isn't a repair")
        }
    }
    pub fn get_reclaim(&self) -> (i32, f32, f32) {
        match &self {
            AllowedAction::Reclaim { reclaim_rate, resource_multiplier, range } => (*reclaim_rate, *resource_multiplier, *range),
            _ => panic!("Isn't a repair")
        }
    }
    pub fn get_range(&self) -> (f32, f32) {
        match &self {
            AllowedAction::Attack(atk) => atk.range,
            AllowedAction::Assist { assist_rate, resource_multiplier, range } => (0.0, *range),
            AllowedAction::Reclaim { reclaim_rate, resource_multiplier, range } => (0.0, *range),
            AllowedAction::Repair { repair_rate, resource_multiplier, range } => (0.0, *range),
            AllowedAction::BuildBuilding { buildings, build_rate, range, placement } => (0.0, *range),
            AllowedAction::BuildUnit { units, build_rate, range, placement } => (0.0, *range)
        }
    }
    pub fn get_build_rate(&self) -> i32 {
        match &self {
            Self::BuildUnit { units, build_rate, range, placement } => *build_rate,
            Self::BuildBuilding { buildings, build_rate, range, placement } => *build_rate,
            _ => panic!("Has no build rate")
        }
    }
    pub fn get_placement(&self) -> Vec3Df {
        match &self {
            Self::BuildUnit { units, build_rate, range, placement } => *placement,
            Self::BuildBuilding { buildings, build_rate, range, placement } => *placement,
            _ => panic!("Has no build rate")
        }
    }
}

#[derive(Clone, Debug)]
pub enum AllowedActions {
    SingleAction(AllowedAction),
    ManyActions(Vec<AllowedAction>)
}

impl AllowedActions {
    pub fn get_building_index(&self) -> usize {

        match &self {
            Self::SingleAction(AllowedAction::BuildBuilding {..}) => 0,
            Self::ManyActions(acts) => acts.iter().enumerate().find(|(i,atk)| {match atk {AllowedAction::BuildBuilding{..} => true, _ => false}}).unwrap().0,
            _ => panic!("No attack in allowed actions")
        }
    }
    pub fn get_attack(&self) -> &Attack {
        match &self {
            Self::SingleAction(AllowedAction::Attack(atk)) => atk,
            Self::ManyActions(acts) => acts.iter().find(|atk| {match atk {AllowedAction::Attack(a) => true, _ => false}}).unwrap().get_attack(),
            _ => panic!("No attack in allowed actions")
        }
    }
    pub fn get_repair_at(&self, at:usize) -> (i32, f32, f32) {
        match &self {
            Self::SingleAction(AllowedAction::Repair { repair_rate, resource_multiplier, range }) => (*repair_rate, *resource_multiplier, *range),
            Self::ManyActions(acts) => acts[at].get_repair(),
            _ => panic!("No repair in allowed actions")
        }
    }
    pub fn get_reclaim_at(&self, at:usize) -> (i32, f32, f32) {
        match &self {
            Self::SingleAction(AllowedAction::Reclaim { reclaim_rate, resource_multiplier, range }) => (*reclaim_rate, *resource_multiplier, *range),
            Self::ManyActions(acts) => acts[at].get_repair(),
            _ => panic!("No repair in allowed actions")
        }
    }
    pub fn get_range_at(&self, at:usize) -> (f32, f32) {
        match &self {
            Self::SingleAction(act) => act.get_range(),
            Self::ManyActions(acts) => acts[at].get_range(),
        }
    }
    pub fn get_build_rate_at(&self, at:usize) -> i32 {
        match &self {
            Self::SingleAction(act) => act.get_build_rate(),
            Self::ManyActions(acts) => acts[at].get_build_rate()
        }
    }
    pub fn get_placement_at(&self, at:usize) -> Vec3Df {
        match &self {
            Self::SingleAction(act) => act.get_placement(),
            Self::ManyActions(acts) => acts[at].get_placement()
        }
    }
}

#[derive(Clone)]
pub struct StaticTurret {
    pub positions:Vec<Vec3Df>,
    pub allowed_actions:AllowedActions,
    pub yaw_range:(f32, f32),
    pub pitch_range:(f32, f32),
    pub max_turn_rate:f32,
    pub max_elevation_rate:f32,
    pub max_health:f32,
    // does it act independently from the unit's actions ? (won't take unit actions)
    pub independent:bool,
    // does it turn before attacking
    pub must_turn:bool,
    pub mesh_info:StaticMeshInfo,
    pub resource_share_cutoff:i64,
}

pub type UnitStaticTypeID = usize;

pub type BuildingStaticTypeID = usize;

#[derive(Clone, Debug)]
pub enum BuildableUnits {
    Specific(Vec<UnitStaticTypeID>)
}

#[derive(Clone, Debug)]
pub enum BuildableBuildings {
    Specific(Vec<BuildingStaticTypeID>)
}

pub type BuildingID = usize;

#[derive(Clone, Debug, PartialEq, FromBytes, ToBytes)]
pub enum TurretActionKind {
    Attack {cast:AttackCast},
    Assist {target:TID, hold:bool},
    Repair {target:TID, hold:bool},
    Reclaim {target:TID},
    BuildUnit {unit_type:UnitStaticTypeID, progress:i32},
    BuildBuilding {building_type:BuildingStaticTypeID, building:Option<BuildingID>, building_space:Vec3Df}
}

#[derive(Clone, Debug, PartialEq, FromBytes, ToBytes)]
pub struct TurretAction {
    kind:TurretActionKind,
    allowed_id:usize,
}

impl TurretAction {
    pub fn new(kind:TurretActionKind, allowed_id:usize) -> Self {
        Self { kind, allowed_id }
    }
}