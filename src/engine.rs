use std::{collections::HashMap, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};

use engine_derive::GameEngine;
use to_from_bytes::{FromBytes, ToBytes};

use hord3::{defaults::default_rendering::vectorinator::{Vectorinator, VectorinatorWrite}, horde::{game_engine::{engine::{GameEngine, MovingObjectID}, entity::{Entity, EntityID, EntityVec, MultiplayerEntity, Renderable}, multiplayer::{GlobalComponent, GlobalEvent, HordeEventReport, HordeMultiModeChoice, HordeMultiplayer, Identify, MultiplayerEngine, MustSync}, world::{World, WorldComputeHandler, WorldEvent, WorldHandler, WorldOutHandler, WorldWriteHandler}}, geometry::vec3d::Vec3Df, rendering::RenderingBackend, scheduler::IndividualTask}};

use super::entity::{CoolEntity, CoolEntityVecRead, CoolEntityVecWrite};
use to_from_bytes_derive::{FromBytes, ToBytes};

#[derive(Clone, ToBytes, FromBytes, PartialEq)]

pub struct SinglePWorld {
    pub test:usize,
}

impl<ID:Identify> WorldEvent<SinglePWorld, ID> for SinglePWorld {
    fn apply_event(self, world:&mut SinglePWorld) {
        
    }
    fn get_source(&self) -> Option<ID> {
        None
    }
    fn should_sync(&self) -> MustSync {
        MustSync::Both
    }
}

impl<ID:Identify> World<ID> for SinglePWorld {
    type RB = TestRB;
    type WE = SinglePWorld;
    fn update_rendering(&mut self, data:&mut <Self::RB as hord3::horde::rendering::RenderingBackend>::PreTickData) {
        
    }
}

impl<'a> Renderable<VectorinatorWrite<'a>> for SinglePWorld {
    fn do_render_changes(&mut self, render_data:&mut VectorinatorWrite<'a>) {
        
    }
}

#[derive(Clone)]
pub struct TestRB {
    lol:usize
}

impl RenderingBackend for TestRB {
    type EntityRenderingData = usize;
    type PreTickData = usize;
    type RenderingStatus = usize;
    type RenderingStatusUpdate = usize;
}


pub fn stage_0<'a>(turn:EntityTurn, id:EntityID, reader: &CoolEntityVecRead<'a, SinglePEngineTID>, world_read: &WorldComputeHandler<'a, SinglePWorld, SinglePEngineTID>) {
    if id == 0 {
        
    }
    println!("TEST COMPUTE");
}


pub fn stage_1<'a>(turn:EntityTurn, id:EntityID, reader: &CoolEntityVecRead<'a, SinglePEngineTID>, world_read: &WorldComputeHandler<'a, SinglePWorld, SinglePEngineTID>) {
    println!("TEST AFTER MAIN");
    match turn {
        EntityTurn::ent1 => {
            
        },
    }
    
}

pub fn stage_2<'a>(turn:EntityTurn, id:EntityID, reader: &CoolEntityVecRead<'a, SinglePEngineTID>, world_read: &WorldComputeHandler<'a, SinglePWorld, SinglePEngineTID>) {
    println!("TEST AFTER MAIN");
    match turn {
        EntityTurn::ent1 => {
            
        },
    }
    
}

#[derive(GameEngine, Clone)]
#[rendering_engine = "Vectorinator"]
#[tick_stages = 3]
pub struct SinglePEngine {
    ent1:CoolEntity,
    world:SinglePWorld,
}