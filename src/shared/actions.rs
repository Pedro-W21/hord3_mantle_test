use hord3::horde::{game_engine::{entity::{Component, ComponentEvent, StaticComponent}, multiplayer::{Identify, MustSync}, static_type_id::HasStaticTypeID, world::{WorldComputeHandler, WorldEvent}}, geometry::vec3d::Vec3Df};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{entities::{building::BuildingVecRead, unit::{UnitEvent, UnitVecRead}}, game_engine::CoolVoxel, game_map::{GameMap, GameMapEvent, VoxelLight, VoxelType, WorldVoxelPos, get_voxel_pos, rts_gen::RTSGen}, shared::{planner::{Plan, PlannerEvent, PlannerUpdate}, position::{PositionEvent, PositionUpdate}, targeting::footprint_available, turrets::{TurretAction, TurretsEvent, TurretsUpdate}}, turbo_game_engine::TurboGameEngineTID};

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub struct Action {
    id:usize,
    started_at:usize, // tick this was started at
    timer:ActionTimer,
    kind:ActionKind,
    source:ActionSource
}
#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum ActionSource {
    Director,
    Planner
}


impl Action {
    pub fn new(id:usize, started_at:usize, timer:ActionTimer, kind:ActionKind, source:ActionSource) -> Self {
        Self { id, started_at, timer, kind, source }
    }
    pub fn get_kind(&self) -> &ActionKind {
        &self.kind
    }
    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn needs_planning(&self) -> bool {
        match &self.kind {
            ActionKind::PathToPosition(pos, tolerance) => true,
            _ => false
        }
    }
    pub fn is_possible<'a>(
        &self,
        agent_id:TID,
        units:&UnitVecRead<'a, TID>,
        buildings:&BuildingVecRead<'a, TID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TID>,
        tick:usize
    ) -> bool {
        match self.kind {
            ActionKind::Jump => true,
            ActionKind::MoveInDirection(_) => true,
            ActionKind::MoveTowards(position, tolerance) => world.world.get_voxel_at(get_voxel_pos(position)).is_some(),
            ActionKind::PathToPosition(position, tolerance) => world.world.get_voxel_at(get_voxel_pos(position)).and_then(|voxel| {if world.world.get_voxel_types()[voxel.voxel_type as usize].is_completely_empty() {Some(true)} else {None}}).is_some(),
            ActionKind::ChangeVoxel(position, _) => world.world.get_chunk_at(world.world.get_chunk_pos_i(position)).is_some(),
            ActionKind::StopAt(pos, _, _) => true,
            ActionKind::BuildBuilding { position, building_type, with_turret } => true,
            _ => true
        }
    }
    pub fn is_done<'a>(
        &self,
        agent_id:TID,
        units:&UnitVecRead<'a, TID>,
        buildings:&BuildingVecRead<'a, TID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TID>,
        tick:usize
    ) -> bool {
        match self.kind {
            ActionKind::Jump => false,
            ActionKind::MoveInDirection(_) => false,
            ActionKind::MoveTowards(position, tolerance) => units.pos[agent_id.unwrap_unit()].pos.dist(&position) < tolerance,
            ActionKind::PathToPosition(position, tolerance) => units.pos[agent_id.unwrap_unit()].pos.dist(&position) < tolerance,
            ActionKind::ChangeVoxel(_, _) => false,
            ActionKind::StopAt(pos, speed_tolerance, pos_tolerance) => {
                let movement = &units.pos[agent_id.unwrap_unit()];
                movement.pos.dist(&pos) < pos_tolerance && movement.spd.norme() < speed_tolerance
            },
            ActionKind::BuildBuilding { position, building_type, with_turret } => !footprint_available(buildings.static_types[building_type].stats.building_footprint + position, buildings, &world.world),
            _ => true
        }
    }
    pub fn perform<'a>(
        &self,
        agent_id:TID,
        units:&UnitVecRead<'a, TID>,
        buildings:&BuildingVecRead<'a, TID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TID>,
        counter:&mut ActionCounter,
        tick:usize
    ) -> ActionResult {
        if self.timer.timed_out(self.started_at, tick) {
            units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent { id: agent_id.unwrap_unit(), source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
            ActionResult::FailedTimer
        }
        else if self.is_possible(agent_id, units, buildings, world, tick) {
            if self.is_done(agent_id, units, buildings, world, tick) {
                units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent { id: agent_id.unwrap_unit(), source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
                ActionResult::Done
            }
            else if self.needs_planning() {
                let planner = &units.planner[agent_id.unwrap_unit()];
                if planner.plan_exists_for(self.id) {
                    let plan = planner.get_plan_for_id(self.id).unwrap();
                    match plan.get_actions_to_add(counter, tick) {
                        Some(actions) => for action in actions.iter().rev() {
                            units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::new(agent_id.unwrap_unit(), None, ActionsUpdate::InsertActionAtStart(action.clone()))));
                        },
                        None => ()
                    }
                    ActionResult::InProgress
                }
                else {
                    match self.kind {
                        ActionKind::PathToPosition(position, tolerance) => {
                            let movement = &units.pos[agent_id.unwrap_unit()];
                            let plan = Plan::create_pathfinding(self.id, tolerance, movement.pos, position, agent_id.unwrap_unit(), 1000, units, buildings, world);
                            let actions = plan.get_actions_to_add(counter, tick);
                            match actions {
                                Some(actions) => for action in actions.iter().rev() {
                                    units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::new(agent_id.unwrap_unit(), None, ActionsUpdate::InsertActionAtStart(action.clone()))));
                                },
                                None => ()
                            }
                            units.tunnels.planner_out.send(UnitEvent::new(MustSync::Both, PlannerEvent::new(agent_id.unwrap_unit(), None, PlannerUpdate::AddPlan(plan))));

                            ActionResult::InProgress
                        },
                        _ => panic!("impossible to get there with an action that requires no planning"),
                    }
                }
                
            }
            else {
                match &self.kind {
                    ActionKind::Jump => {
                        let movement = &units.pos[agent_id.unwrap_unit()];
                        let stats = &units.stats[agent_id.unwrap_unit()];
                        let static_stats = &units.static_types[stats.get_id()];
                        if movement.touching_ground {
                            units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(agent_id.unwrap_unit(), None, PositionUpdate::AddToSpeed(Vec3Df::new(0.0, 0.0, static_stats.pos.jump_height)))));
                            units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent { id: agent_id.unwrap_unit(), source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
                            ActionResult::Done
                        }
                        else {
                            units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent { id: agent_id.unwrap_unit(), source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
                            ActionResult::Error(ActionError::ImpossibleAction)
                        }
                    },
                    ActionKind::MoveInDirection(direction) => {
                        let movement = &units.pos[agent_id.unwrap_unit()];
                        let stats = &units.stats[agent_id.unwrap_unit()];
                        let static_stats = &units.static_types[stats.get_id()];
                        if direction.z != 0.0 && movement.against_wall && movement.touching_ground {
                            units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(agent_id.unwrap_unit(), None, PositionUpdate::AddToSpeed(Vec3Df::new(direction.x * static_stats.pos.ground_speed, direction.y * static_stats.pos.ground_speed, static_stats.pos.jump_height)))));
                        }
                        else {
                            units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(agent_id.unwrap_unit(), None, PositionUpdate::AddToSpeed(Vec3Df::new(direction.x * static_stats.pos.ground_speed, direction.y * static_stats.pos.ground_speed, 0.0)))));
                        }
                        units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::new(agent_id.unwrap_unit(), None, ActionsUpdate::RemoveAction(self.id))));
                        ActionResult::InProgress
                    },
                    ActionKind::MoveTowards(position, tolerance) => {
                        let movement = &units.pos[agent_id.unwrap_unit()];
                        let stats = &units.stats[agent_id.unwrap_unit()];
                        let static_stats = &units.static_types[stats.get_id()];
                        let mut direction = position - movement.pos;
                        direction.z = 0.0;
                        direction = direction.normalise();
                        if movement.against_wall {
                            units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(agent_id.unwrap_unit(), None, PositionUpdate::AddToSpeed(Vec3Df::new(direction.x * static_stats.pos.ground_speed, direction.y * static_stats.pos.ground_speed, static_stats.pos.jump_height)))));
                        }
                        else {
                            units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(agent_id.unwrap_unit(), None, PositionUpdate::AddToSpeed(Vec3Df::new(direction.x * static_stats.pos.ground_speed, direction.y * static_stats.pos.ground_speed, 0.0)))));
                        }
                        units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::new(agent_id.unwrap_unit(), None, ActionsUpdate::RemoveAction(self.id))));
                        ActionResult::InProgress
                    },
                    ActionKind::StopAt(pos, spd_tolerance, pos_tolerance) => {
                        
                        let movement = &units.pos[agent_id.unwrap_unit()];
                        let dist = movement.pos.dist(&pos);
                        if dist >= *pos_tolerance {
                            let mut direction = pos - movement.pos;
                            let stats = &units.stats[agent_id.unwrap_unit()];
                            let static_stats = &units.static_types[stats.get_id()];
                            direction.z = 0.0;
                            direction = direction.normalise();
                            if movement.against_wall {
                                units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(agent_id.unwrap_unit(), None, PositionUpdate::AddToSpeed(Vec3Df::new(direction.x * static_stats.pos.ground_speed * 0.2, direction.y * static_stats.pos.ground_speed * 0.2, static_stats.pos.jump_height)))));
                            }
                            else {
                                units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(agent_id.unwrap_unit(), None, PositionUpdate::AddToSpeed(Vec3Df::new(direction.x * static_stats.pos.ground_speed * 0.2, direction.y * static_stats.pos.ground_speed * 0.2, 0.0)))));
                            }
                        }
                        else {
                            let mut direction = -movement.spd;
                            let stats = &units.stats[agent_id.unwrap_unit()];
                            let static_stats = &units.static_types[stats.get_id()];
                            direction.z = 0.0;
                            direction = direction.normalise();
                            units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(agent_id.unwrap_unit(), None, PositionUpdate::AddToSpeed(Vec3Df::new(direction.x * static_stats.pos.ground_speed * 0.5, direction.y * static_stats.pos.ground_speed * 0.5, 0.0)))));
                        }

                        ActionResult::InProgress
                    }
                    ActionKind::ChangeVoxel(voxel_pos, new_voxel) => {
                        world.tunnels.send_event(GameMapEvent::UpdateVoxelAt(voxel_pos.clone(), new_voxel.clone()));
                        units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::new(agent_id.unwrap_unit(), None, ActionsUpdate::RemoveAction(self.id))));
                        ActionResult::Done
                    },
                    ActionKind::BuildBuilding { position, building_type,with_turret } => {
                        let static_turret = &units.static_types[units.stats[agent_id.unwrap_unit()].get_id()];
                        match with_turret {
                            None => {
                                // println!("ADDING BUILDING TO {:?}", agent_id);
                                let allowed_id = static_turret.turrets.turrets[0].allowed_actions.get_building_index();
                                let action = TurretAction::new(super::turrets::TurretActionKind::BuildBuilding { building_type:*building_type, building: None, building_space: *position }, allowed_id);
                                units.tunnels.turrets_out.send(UnitEvent::new(MustSync::Both, TurretsEvent::new(agent_id.unwrap_unit(), None, TurretsUpdate::AddSingleAction(action))));
                            
                            },
                            Some(turret) => {
                                let turrets = &units.turrets[agent_id.unwrap_unit()].turrets;
                                let allowed_id = static_turret.turrets.turrets[turrets.unwrap_multi_ref()[*turret].static_id].allowed_actions.get_building_index();
                                let action = TurretAction::new(super::turrets::TurretActionKind::BuildBuilding { building_type:*building_type, building: None, building_space: *position }, allowed_id);
                                let mut actions = vec![None ; turrets.unwrap_multi_ref().iter().len()];
                                actions[*turret] = Some(action);
                                units.tunnels.turrets_out.send(UnitEvent::new(MustSync::Both, TurretsEvent::new(agent_id.unwrap_unit(), None, TurretsUpdate::AddMultiAction(actions))));
                            }
                        }
                        units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent { id: agent_id.unwrap_unit(), source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
                        ActionResult::Done
                    },
                    ActionKind::PathToPosition(position, tolerance) => ActionResult::Error(ActionError::ImpossibleAction),
                    _ => ActionResult::Error(ActionError::ImpossibleAction)
                }
            }
        }
        else {
            units.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent { id: agent_id.unwrap_unit(), source: None, variant: ActionsUpdate::RemoveAction(self.id)}));
            ActionResult::Error(ActionError::ImpossibleAction)
        }
    }

}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum ActionTimer {
    Infinite,
    Delay(usize), // ticks
    Deadline(usize) // must be done by specified tick
}

impl ActionTimer {
    pub fn timed_out(&self, started:usize, current_tick:usize) -> bool {
        match self {
            ActionTimer::Infinite => false,
            ActionTimer::Delay(delay) => current_tick - started >= *delay,
            ActionTimer::Deadline(deadline) => current_tick >= *deadline,

        }
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum ActionResult {
    Done,
    InProgress,
    FailedTimer,
    Error(ActionError)
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum ActionError {
    ImpossibleAction,
    StartedAfterDeadline
}


pub type TID = TurboGameEngineTID;


#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum AttackTarget {
    Entity(TID),
    Position(Vec3Df)
}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum ActionKind {
    MoveInDirection(Vec3Df),
    Jump,
    PathToPosition(Vec3Df, f32),
    MoveTowards(Vec3Df, f32),
    StopAt(Vec3Df, f32, f32),
    ChangeVoxel(WorldVoxelPos, CoolVoxel),
    Attack {
        target:AttackTarget
    },
    Assist {
        target:TID,
        hold:bool,
    },
    Repair {
        target:TID,
        hold:bool,
    },
    Reclaim {
        target:TID,
    },
    BuildBuilding {
        position:Vec3Df,
        building_type:usize,
        with_turret:Option<usize>
    }

}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub struct ActionCounter {
    latest_id:usize,
}

impl ActionCounter {
    pub fn new() -> Self {
        Self { latest_id: 0 }
    }
    pub fn get_next_id(&mut self) -> usize {
        let id = self.latest_id;
        self.latest_id += 1;
        id
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct Actions {
    action_counter:ActionCounter,
    all_actions:Vec<Action>
}

impl Actions {
    pub fn get_counter(&self) -> &ActionCounter {
        &self.action_counter
    }
    pub fn new() -> Self {
        Self { action_counter: ActionCounter::new(), all_actions: Vec::with_capacity(8) }
    }
    pub fn len(&self) -> usize {
        self.all_actions.len()
    }
    pub fn perform<'a>(
        &self,
        agent_id:TID,
        units:&UnitVecRead<'a, TID>,
        buildings:&BuildingVecRead<'a, TID>,
        world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TID>,
        counter:&mut ActionCounter,
        tick:usize
    ) {
        if self.all_actions.len() > 0 {
            let action = &self.all_actions[0];
            let result = action.perform(agent_id, units, buildings, world, counter, tick);
            match &result {
                ActionResult::InProgress => (),
                _ => match action.source {
                    ActionSource::Planner => (),//units.tunnels.planner_out.send(PlannerEvent::new(agent_id.unwrap_unit(), None, PlannerUpdate::AddFinished((action.clone(), result)))).unwrap(),
                    ActionSource::Director => ()//units.tunnels.director_out.send(DirectorEvent::new(agent_id.unwrap_unit(), None, DirectorUpdate::NotifyFinished((action.clone(), result)))).unwrap(),
                }
            }
        }
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StaticActions {
    pub base_actions:Vec<Action>
}

impl StaticComponent for StaticActions {
    
}
#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct ActionsEvent<ID:Identify> {
    id:usize,
    source:Option<ID>,
    variant:ActionsUpdate
}

impl<ID:Identify> ActionsEvent<ID> {
    pub fn new(id:usize, source:Option<ID>, variant:ActionsUpdate) -> Self {
        Self { id, source, variant }
    }
}

impl<ID:Identify> ComponentEvent<Actions, ID> for ActionsEvent<ID> {
    type ComponentUpdate = ActionsUpdate;
    fn get_id(&self) -> hord3::horde::game_engine::entity::EntityID {
        self.id
    }
    fn get_source(&self) -> Option<ID> {
        self.source.clone()
    }
    fn apply_to_component(self, components:&mut Vec<Actions>) {
        match self.variant {
            ActionsUpdate::UpdateCounter(counter) => components[self.id].action_counter = counter,
            ActionsUpdate::AddAction(action) => components[self.id].all_actions.push(action),
            ActionsUpdate::RemoveAction(action_id) => {
                match components[self.id].all_actions.iter().enumerate().find(|(i, action)| {action.id == action_id}) {
                    Some((i, action)) => {
                        components[self.id].all_actions.remove(i);
                    },
                    None => ()
                }
            },
            ActionsUpdate::InsertActionAtStart(action) => components[self.id].all_actions.insert(0,action),
            ActionsUpdate::UpdateAllActions(new_actions) => components[self.id].all_actions = new_actions,
        }
    }  
}
#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum ActionsUpdate {
    UpdateCounter(ActionCounter),
    AddAction(Action),
    InsertActionAtStart(Action),
    UpdateAllActions(Vec<Action>),
    RemoveAction(usize) // action id
}

impl<ID:Identify> Component<ID> for Actions {
    type SC = StaticActions;
    type CE = ActionsEvent<ID>;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { action_counter: ActionCounter::new(), all_actions:static_comp.base_actions.clone() }
    }
}