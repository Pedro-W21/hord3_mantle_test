use std::{collections::HashMap, sync::{Arc, atomic::{AtomicUsize, Ordering}}};

use engine_derive::GameEngine;

use crate::{entities::{building::{Building, BuildingEvent, BuildingVecRead, BuildingVecWrite}, projectile::{Projectile, ProjectileEvent, ProjectileVecRead, ProjectileVecWrite, projectile_stats::{ProjectileAOE, ProjectileDeath, ProjectileStatsEvent, ProjectileStatsUpdate}}, reclaim::{Reclaim, ReclaimEvent, ReclaimVecRead, ReclaimVecWrite, simple_stats::{SimpleStatsEvent, SimpleStatsUpdate}}, team::{RenderTeam, Team, TeamEvent, TeamVecRead, TeamVecWrite}, unit::{Unit, UnitEvent, UnitVecRead, UnitVecWrite}}, game_engine::CoolVoxel, game_entity::GameEntityVecRead, game_map::{GameMap, GameMapEvent, VoxelGetHint, VoxelLight, VoxelType, get_voxel_pos, raycaster::Ray, rts_gen::RTSGen}, shared::{actions::TID, colliders::{Collision, ComplexCollideEvent, ComplexCollider, ComplexColliderUpdate}, consts::BUILDING_SET, position::{PositionEvent, PositionUpdate}, resources::{TeamResourceEvent, TeamResourcesUpdate}, stats::{StatsEvent, StatsUpdate}, targeting::{TargetEnvironment, damage_entity, do_death_accounting, get_air_units_in_range_with_diff_team, get_buildings_in_range_with_diff_team, get_grid_index_for, get_ground_units_in_range_with_diff_team, get_naval_units_in_range_with_diff_team, get_projectiles_in_range_with_diff_team, get_reclaim_in_range, remove_resource_contribs_of}}};
use hord3::{defaults::default_rendering::{mantle::api::{MantleEvent, MantleHandler}, vectorinator_binned::{Vectorinator, rendering_spaces::ViewportData, shaders::NoOpShader}}, horde::{game_engine::{engine::{GameEngine, MovingObjectID}, entity::{Entity, EntityVec, MultiplayerEntity, Renderable}, multiplayer::{GlobalComponent, GlobalEvent, HordeEventReport, HordeMultiModeChoice, HordeMultiplayer, Identify, MultiplayerEngine, MustSync}, static_type_id::HasStaticTypeID, world::{World, WorldComputeHandler, WorldEvent, WorldHandler, WorldOutHandler, WorldWriteHandler}}, geometry::vec3d::{Vec3D, Vec3Df}, rendering::camera::Camera, scheduler::IndividualTask, sound::{ARWWaves, WavesHandler}}};

pub fn test() {
    
}

#[derive(Clone)]
pub struct ExtraData {
    pub tick:Arc<AtomicUsize>,
    pub waves:WavesHandler<TurboGameEngine>,
    pub mantle:MantleHandler,
}

/// Resource accounting
pub fn stage_0<'a>(turn:EntityTurn, id:usize, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>, teams:&TeamVecRead<'a, TurboGameEngineTID>, world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TurboGameEngineTID>, extra_data:&ExtraData) {
    match turn {
        EntityTurn::units => if units.stats[id].active() {

            let pos = &units.pos[id];
            let turrets = &units.turrets[id];
            let stats = &units.stats[id];
            let static_type = &units.static_types[stats.get_id()];
            turrets.first_stage(TID::units(id), pos.pos, &static_type.turrets, units, buildings, reclaim, projectiles, teams);
        },
        EntityTurn::buildings => if buildings.stats[id].active() {

            let pos = &buildings.pos[id];
            let turrets = &buildings.turrets[id];
            let stats = &buildings.stats[id];
            let static_type = &buildings.static_types[stats.get_id()];
            turrets.first_stage(TID::buildings(id), pos.pos, &static_type.turrets, units, buildings, reclaim, projectiles, teams);
        },
        _ => ()
    }
}

/// activating/deactivating based on resources when needed, doing basic pos, turrets get new actions if needed
pub fn stage_1<'a>(turn:EntityTurn, id:usize, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>, teams:&TeamVecRead<'a, TurboGameEngineTID>, world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TurboGameEngineTID>, extra_data:&ExtraData) {
    match turn {
        EntityTurn::units => if units.stats[id].active() {

            let pos = &units.pos[id];
            let turrets = &units.turrets[id];
            let stats = &units.stats[id];
            let static_type = &units.static_types[stats.get_id()];
            let mut targeting_data = None;
            turrets.second_stage(&mut targeting_data, TID::units(id), pos.pos, stats.team, &static_type.turrets, units, buildings, reclaim, projectiles, &world.world.set_grid);

            let actions = &units.actions[id];
            let mut counter = actions.get_counter().clone();
            actions.perform(TID::units(id), units, buildings, world, &mut counter, extra_data.tick.load(Ordering::Relaxed));
        },
        EntityTurn::buildings => if buildings.stats[id].active() {
            // TODO : implement building actions
            let pos = &buildings.pos[id];
            let turrets = &buildings.turrets[id];
            let stats = &buildings.stats[id];
            let static_type = &buildings.static_types[stats.get_id()];
            let mut targeting_data = None;
            turrets.second_stage(&mut targeting_data, TID::buildings(id), pos.pos, stats.team, &static_type.turrets, units, buildings, reclaim, projectiles, &world.world.set_grid);

        },
        EntityTurn::projectiles => (),
        EntityTurn::reclaim => (),
        EntityTurn::teams => (),
    }
}

/// do turret actions, projectile/entity collisions and units/buildings collisions
pub fn stage_2<'a>(turn:EntityTurn, id:usize, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>, teams:&TeamVecRead<'a, TurboGameEngineTID>, world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TurboGameEngineTID>, extra_data:&ExtraData) {
    match turn {
        EntityTurn::units => if units.stats[id].active()  {
            let pos = &units.pos[id];
            let turrets = &units.turrets[id];
            let stats = &units.stats[id];
            let static_type = &units.static_types[stats.get_id()];
            turrets.third_stage(extra_data.tick.load(Ordering::Relaxed), TID::units(id), pos.pos, &static_type.turrets, units, buildings, reclaim, projectiles, teams, &world);
        
        },
        EntityTurn::buildings => if buildings.stats[id].active(){
            let pos = &buildings.pos[id];
            let turrets = &buildings.turrets[id];
            let stats = &buildings.stats[id];
            let static_type = &buildings.static_types[stats.get_id()];
            turrets.third_stage(extra_data.tick.load(Ordering::Relaxed), TID::buildings(id), pos.pos, &static_type.turrets, units, buildings, reclaim, projectiles, teams, &world);
        
        },
        EntityTurn::projectiles => if !projectiles.stats[id].dead {
            let pos = &projectiles.pos[id];
            let stats = &projectiles.stats[id];
            let static_type = &projectiles.static_types[stats.get_id()];
            let self_collider = &projectiles.collider[id];
            let mut collided = None;
            'envs: for env in &static_type.stats.target_environments {
                match env {
                    TargetEnvironment::Ground => {
                        let potential = get_ground_units_in_range_with_diff_team(stats.team, pos.spd.norme(), pos.pos, &world.world.set_grid, units);
                        for ground_unit in potential {
                            let collider = &units.collider[ground_unit];
                            match collider.get_collision_with(&self_collider) {
                                Collision::Yes => match collided {
                                    Some(tid) => (),
                                    None => {
                                        // TODO : handle piercing ?
                                        collided = Some(TID::units(ground_unit));
                                        break 'envs;
                                    }
                                },
                                Collision::No => ()
                            }
                        }
                    },
                    TargetEnvironment::Building => {
                        let potential = get_buildings_in_range_with_diff_team(stats.team, pos.spd.norme(), pos.pos, &world.world.set_grid, buildings);
                        for building in potential {
                            let collider = &buildings.collider[building];
                            match collider.get_collision_with(&self_collider) {
                                Collision::Yes => match collided {
                                    Some(tid) => (),
                                    None => {
                                        // TODO : handle piercing ?
                                        collided = Some(TID::buildings(building));
                                        break 'envs;
                                    }
                                },
                                Collision::No => ()
                            }
                        }
                    },
                    TargetEnvironment::Air => {
                        let potential = get_air_units_in_range_with_diff_team(stats.team, pos.spd.norme(), pos.pos, &world.world.set_grid, units);
                        for air_unit in potential {
                            let collider = &units.collider[air_unit];
                            match collider.get_collision_with(&self_collider) {
                                Collision::Yes => match collided {
                                    Some(tid) => (),
                                    None => {
                                        // TODO : handle piercing ?
                                        collided = Some(TID::units(air_unit));
                                        break 'envs;
                                    }
                                },
                                Collision::No => ()
                            }
                        }
                    },
                    TargetEnvironment::Naval => {
                        let potential = get_naval_units_in_range_with_diff_team(stats.team, pos.spd.norme(), pos.pos, &world.world.set_grid, units);
                        for naval_unit in potential {
                            let collider = &units.collider[naval_unit];
                            match collider.get_collision_with(&self_collider) {
                                Collision::Yes => match collided {
                                    Some(tid) => (),
                                    None => {
                                        // TODO : handle piercing ?
                                        collided = Some(TID::units(naval_unit));
                                        break 'envs;
                                    }
                                },
                                Collision::No => ()
                            }
                        }
                    },
                    TargetEnvironment::Projectile => {
                        let potential = get_projectiles_in_range_with_diff_team(stats.team, pos.spd.norme(), pos.pos, &world.world.set_grid, projectiles);
                        for ground_unit in potential {
                            let collider = &projectiles.collider[ground_unit];
                            match collider.get_collision_with(&self_collider) {
                                Collision::Yes => match collided {
                                    Some(tid) => (),
                                    None => {
                                        // TODO : handle piercing ?
                                        collided = Some(TID::units(ground_unit));
                                        break 'envs;
                                    }
                                },
                                Collision::No => ()
                            }
                        }
                    },
                    TargetEnvironment::Reclaim => {
                        let potential = get_reclaim_in_range(pos.spd.norme(), pos.pos, &world.world.set_grid, reclaim);
                        for debris in potential {
                            let collider = &reclaim.collider[debris];
                            match collider.get_collision_with(&self_collider) {
                                Collision::Yes => match collided {
                                    Some(tid) => (),
                                    None => {
                                        // TODO : handle piercing ?
                                        collided = Some(TID::reclaim(debris));
                                        break 'envs;
                                    }
                                },
                                Collision::No => ()
                            }
                        }
                    },   
                }
            }
            match collided {
                Some(collision) => match static_type.stats.on_death {
                    ProjectileDeath::Simple => {
                        println!("Projectile {id} hit {collision:?}");
                        match &static_type.stats.aoe {
                            ProjectileAOE::DirectDamage { range, damage } => damage_entity(damage.get_simple_damage(), TID::projectiles(id), collision, units, buildings, projectiles, reclaim),
                            _ => todo!("Implement non-direct damage")
                        }
                        damage_entity(projectiles.health[id].current_health, TID::projectiles(id), TID::projectiles(id), units, buildings, projectiles, reclaim);
                    },
                    _ => todo!("Implement other projectile deaths")
                },
                None => ()
            }
        },
        EntityTurn::reclaim => (),
        EntityTurn::teams => {
            
        }
    }
}

fn get_nudge_to_nearest_next_whole(number:f32, delta_to_add:f32) -> f32 {
    if number.is_sign_positive() {
        let fract = number.fract();
        let nudge = if fract >= 0.5 {
            1.0 - fract + delta_to_add
        }
        else {
            -fract - delta_to_add
        };
        nudge
    }
    else {
        let fract = number.fract();
        let nudge = if fract <= -0.5 {
            -1.0 - fract - delta_to_add
        }
        else {
            -fract + delta_to_add
        };
        nudge
    }
}

fn compute_nudges_from(vertex:Vec3Df, spd:Vec3Df, collider:&ComplexCollider, world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TID>, hint:&mut Option<VoxelGetHint>) -> (Vec3Df, bool) {
    let mut touching_ground = false;
    let mut nudges = Vec3Df::all_ones();
    let voxel = match world.world.get_voxel_at_hinted(get_voxel_pos(vertex), hint) {
        Some(voxel) => {
            voxel.clone()
        },
        None => {CoolVoxel::new(0, 0, VoxelLight::zero_light(), 0,None)}
    };
    if !world.world.get_voxel_types()[voxel.voxel_type as usize].is_completely_empty() {    
        let z_nudge = get_nudge_to_nearest_next_whole(vertex.z, 0.01);
        let z_nudged_collider = (*collider.get_global_aabb() + spd + Vec3Df::new(0.0, 0.0, z_nudge));
        if z_nudged_collider.collision_world_hinted(&world.world, hint) {
            let x_nudge = get_nudge_to_nearest_next_whole(vertex.x, 0.01);
            let y_nudge = get_nudge_to_nearest_next_whole(vertex.y, 0.01);
            let mut one_worked = true;
            if x_nudge.abs() < y_nudge.abs() {
                let x_nudged_collider = (*collider.get_global_aabb() + spd + Vec3Df::new(x_nudge, 0.0, 0.0));
                if x_nudged_collider.collision_world_hinted(&world.world, hint) {
                    one_worked = false;
                }
                else {
                    nudges.x = x_nudge;
                    nudges.z = 0.0;
                    nudges.y = 0.0;
                }
            }
            else {
                let y_nudged_collider = (*collider.get_global_aabb() + spd + Vec3Df::new(0.0, y_nudge, 0.0));
                if y_nudged_collider.collision_world_hinted(&world.world, hint) {
                    one_worked = false;
                }
                else {
                    nudges.y = y_nudge;
                    nudges.x = 0.0;
                    nudges.z = 0.0;
                }
            }
            if !one_worked {
                let xy_nudged_collider = (*collider.get_global_aabb() + spd + Vec3Df::new(x_nudge, y_nudge, 0.0));
                if xy_nudged_collider.collision_world_hinted(&world.world, hint) {
                    nudges.x = x_nudge;
                    nudges.y = y_nudge;
                    nudges.z = z_nudge;
                }
                else {
                    nudges.x = x_nudge;
                    nudges.y = y_nudge;
                    nudges.z = 0.0
                }
            }
        }
        else {
            touching_ground = true;
            nudges.z = z_nudge;
            nudges.x = 0.0;
            nudges.y = 0.0;
        }   
    }
    (nudges, touching_ground)
}


fn compute_nudges_from_simple(vertex:Vec3Df, spd:Vec3Df, world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TID>, hint:&mut Option<VoxelGetHint>) -> (Vec3Df, bool) {
    let mut touching_ground = false;
    let mut nudges = Vec3Df::all_ones();
    let voxel = match world.world.get_voxel_at_hinted(get_voxel_pos(vertex), hint) {
        Some(voxel) => {
            voxel.clone()
        },
        None => {CoolVoxel::new(0, 0, VoxelLight::zero_light(), 0,None)}
    };
    if !world.world.get_voxel_types()[voxel.voxel_type as usize].is_completely_empty() {    
        let z_nudge = get_nudge_to_nearest_next_whole(vertex.z, 0.01);
        let z_nudged_collider = (vertex + spd + Vec3Df::new(0.0, 0.0, z_nudge));
        if world.world.simple_collision_hinted(z_nudged_collider, hint) {
            let x_nudge = get_nudge_to_nearest_next_whole(vertex.x, 0.01);
            let y_nudge = get_nudge_to_nearest_next_whole(vertex.y, 0.01);
            let mut one_worked = true;
            if x_nudge.abs() < y_nudge.abs() {
                let x_nudged_collider = (vertex + spd + Vec3Df::new(x_nudge, 0.0, 0.0));
                if world.world.simple_collision_hinted(x_nudged_collider, hint) {
                    one_worked = false;
                }
                else {
                    nudges.x = x_nudge;
                    nudges.z = 0.0;
                    nudges.y = 0.0;
                }
            }
            else {
                let y_nudged_collider = (vertex + spd + Vec3Df::new(0.0, y_nudge, 0.0));
                if world.world.simple_collision_hinted(y_nudged_collider, hint) {
                    one_worked = false;
                }
                else {
                    nudges.y = y_nudge;
                    nudges.x = 0.0;
                    nudges.z = 0.0;
                }
            }
            if !one_worked {
                let xy_nudged_collider = (vertex + spd + Vec3Df::new(x_nudge, y_nudge, 0.0));
                if  world.world.simple_collision_hinted(xy_nudged_collider, hint) {
                    nudges.x = x_nudge;
                    nudges.y = y_nudge;
                    nudges.z = z_nudge;
                }
                else {
                    nudges.x = x_nudge;
                    nudges.y = y_nudge;
                    nudges.z = 0.0
                }
            }
        }
        else {
            touching_ground = true;
            nudges.z = z_nudge;
            nudges.x = 0.0;
            nudges.y = 0.0;
        }   
    }
    (nudges, touching_ground)
}


pub const GRAVITY:f32 = 9.81/180.0;
const AIR_RESISTANCE:f32 = 0.99;

/// check if health <= 0, if so do death and death accounting
/// otherwise do world collision
pub fn stage_3<'a>(turn:EntityTurn, id:usize, units:&UnitVecRead<'a, TurboGameEngineTID>, buildings:&BuildingVecRead<'a, TurboGameEngineTID>, projectiles:&ProjectileVecRead<'a, TurboGameEngineTID>, reclaim:&ReclaimVecRead<'a, TurboGameEngineTID>, teams:&TeamVecRead<'a, TurboGameEngineTID>, world:&WorldComputeHandler<GameMap<CoolVoxel, RTSGen>, TurboGameEngineTID>, extra_data:&ExtraData) {
    match turn {
        EntityTurn::units => if units.stats[id].active() {
            if units.health[id].is_dead() {
                remove_resource_contribs_of(TID::units(id), units, buildings, teams);
                do_death_accounting(TID::units(id), units, buildings, projectiles, reclaim);
                let stats = &units.stats[id];
                let static_stats = &units.static_types[stats.get_id()];
                units.mesh_info[id].free_meshes(&static_stats.mesh_info, &static_stats.turrets, &extra_data.mantle);
                units.tunnels.stats_out.send(UnitEvent::new(MustSync::Both, StatsEvent::new(id, None, StatsUpdate::Kill)));
                let pos = &units.pos[id];
                match world.world.set_grid.get_point_move_update(&pos.pos, &(Vec3Df::all_ones() * 100_000.0), id, get_grid_index_for(stats.current_env)) {
                    Some(update) => {
                        world.tunnels.send_event(GameMapEvent::UpdateSetGrid(update));
                    },
                    None => ()
                }
                units.kill_ent(id);
                // TODO : special deaths (explosion AOEs)
            }
            else {
                let pos = &units.pos[id];
                let turrets = &units.turrets[id];
                let stats = &units.stats[id];
                let static_type = &units.static_types[stats.get_id()];
                turrets.fourth_stage(TID::units(id), stats.team, pos.pos, &static_type.turrets, units, buildings, reclaim, projectiles, teams);

                let collider = &units.collider[id];
                let stats = &units.stats[id];
                let mut movement_pos = pos.pos;
                let mut spd = pos.spd;
                spd *= AIR_RESISTANCE;
                spd.z -= GRAVITY;
                if spd.z.abs() > 0.45 {
                    spd.z = 0.45 * spd.z.signum()
                }
                if spd.x.abs() > 0.45 {
                    spd.x = 0.45 * spd.x.signum()
                }
                if spd.y.abs() > 0.45 {
                    spd.y = 0.45 * spd.y.signum()
                }
                let mut touching_ground = false;
                let mut against_wall = false;
                //dbg!(collider.get_global_aabb());
                // let moved_aabb = (*collider.get_global_aabb() + movement_pos + spd);
                let mut smallest_nudge = Vec3Df::all_ones();
                let mut hint = pos.voxel_hint.clone();
                {
                    let mut vertex = movement_pos;
                    vertex.z += collider.get_global_aabb().get_first_point().z;
                    let (nudges, vertical) = compute_nudges_from_simple(vertex, spd, world, &mut hint);
                    touching_ground = touching_ground | vertical;
                    if touching_ground {
                        if nudges.z.abs() < smallest_nudge.z.abs() {
                            smallest_nudge = nudges;
                        }
                    }
                    else {
                        if nudges.norme_square() < smallest_nudge.norme_square() {
                            smallest_nudge = nudges;
                        }
                    }
                }

                if !touching_ground {
                        let mut vertex = movement_pos;
                        vertex.z += collider.get_global_aabb().get_second_point().z;
                        let (nudges, vertical) = compute_nudges_from_simple(vertex, spd, world, &mut hint);
                        if nudges.norme_square() < smallest_nudge.norme_square() {
                            smallest_nudge = nudges;
                        }
                }
                /*for vertex in moved_aabb.get_ground_vertices() {
                    let (nudges, vertical) = compute_nudges_from(vertex, spd, collider, world, &mut hint);
                    touching_ground = touching_ground | vertical;
                    if touching_ground {
                        if nudges.z.abs() < smallest_nudge.z.abs() {
                            smallest_nudge = nudges;
                        }
                    }
                    else {
                        if nudges.norme_square() < smallest_nudge.norme_square() {
                            smallest_nudge = nudges;
                        }
                    }
                }
                if !touching_ground {
                    for vertex in moved_aabb.get_top_vertices() {
                        let (nudges, vertical) = compute_nudges_from(vertex, spd, collider, world, &mut hint);
                        if nudges.norme_square() < smallest_nudge.norme_square() {
                            smallest_nudge = nudges;
                        }
                    }
                }*/
                if smallest_nudge != Vec3Df::all_ones() {
                    if smallest_nudge.x != 0.0 || smallest_nudge.y != 0.0 && smallest_nudge.z > 0.0 {
                        against_wall = true;
                    }
                    spd += smallest_nudge;
                }
            
                match world.world.set_grid.get_point_move_update(&pos.pos, &(movement_pos + spd), id, get_grid_index_for(stats.current_env)) {
                    Some(update) => {
                        //dbg!(update.clone());
                        world.tunnels.send_event(GameMapEvent::UpdateSetGrid(update))
                    },
                    None => ()
                }
                if touching_ground != pos.touching_ground {
                    units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(id, None, PositionUpdate::UpdateTouchingGround(touching_ground))));
                }
                if against_wall != pos.against_wall {
                    units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(id, None, PositionUpdate::UpdateAgainstWall(against_wall))));
                }


                units.tunnels.pos_out.send(UnitEvent::new(MustSync::Both, PositionEvent::new(id, None, PositionUpdate::UpdatePosAndMoveSpeedAndHint(movement_pos + spd, spd, hint))));
                //units.tunnels.collider_out.send(UnitEvent::new(MustSync::Both, ComplexCollideEvent::new(id, None, ComplexColliderUpdate::UpdateAABB(*static_type.collider.get_global_aabb() + (movement_pos + spd )))));
            


                // entity/world collisions
            }
        },
        EntityTurn::buildings => if buildings.stats[id].active() {
            if buildings.health[id].is_dead() {
                remove_resource_contribs_of(TID::buildings(id), units, buildings, teams);
                do_death_accounting(TID::buildings(id), units, buildings, projectiles, reclaim);
                let static_stats = &buildings.static_types[buildings.stats[id].get_id()];
                buildings.mesh_info[id].free_meshes(&static_stats.mesh_info, &static_stats.turrets, &extra_data.mantle);
                buildings.tunnels.stats_out.send(BuildingEvent::new(MustSync::Both, StatsEvent::new(id, None, StatsUpdate::Kill)));
                let pos = &buildings.pos[id];
                match world.world.set_grid.get_point_move_update(&pos.pos, &(Vec3Df::all_ones() * 100_000.0), id, BUILDING_SET) {
                    Some(update) => {
                        world.tunnels.send_event(GameMapEvent::UpdateSetGrid(update));
                    },
                    None => ()
                }
                buildings.kill_ent(id);
                // TODO : special deaths (explosions AOEs)
            }
            else {

                let pos = &buildings.pos[id];
                let turrets = &buildings.turrets[id];
                let stats = &buildings.stats[id];
                let static_type = &buildings.static_types[stats.get_id()];
                turrets.fourth_stage(TID::buildings(id), stats.team, pos.pos, &static_type.turrets, units, buildings, reclaim, projectiles, teams);
            }
        },
        EntityTurn::projectiles => if !projectiles.stats[id].dead {
            if projectiles.health[id].is_dead() || projectiles.stats[id].ttl_finished(extra_data.tick.load(Ordering::Relaxed)) {
                // println!("Projectile {id} died of TTL");
                do_death_accounting(TID::projectiles(id), units, buildings, projectiles, reclaim);
                let static_stats = &projectiles.static_types[projectiles.stats[id].get_id()];
                projectiles.mesh_info[id].free_mesh(&static_stats.mesh_info, &extra_data.mantle);
                projectiles.tunnels.stats_out.send(ProjectileEvent::new(MustSync::Both, ProjectileStatsEvent::new(id, None, ProjectileStatsUpdate::Kill)));
                projectiles.kill_ent(id);
                // TODO : OnDeath things
            }
            else {
                const MAX_LENGTH:f32 = 1.0;
                let pos = &projectiles.pos[id];
                let mut new_hint = pos.voxel_hint.clone();
                let ray = Ray::new(pos.pos, pos.spd, Some(MAX_LENGTH));
                let end = ray.get_end(&world.world, &mut new_hint);
                if end.final_length < MAX_LENGTH {
                    // hit the world
                    // println!("Projectile {id} hit world at len {} and pos {:?} with speed {:?}", end.final_length, end.end, pos.spd);
                    do_death_accounting(TID::projectiles(id), units, buildings, projectiles, reclaim);
                    let static_stats = &projectiles.static_types[projectiles.stats[id].get_id()];
                    projectiles.mesh_info[id].free_mesh(&static_stats.mesh_info, &extra_data.mantle);
                    projectiles.tunnels.stats_out.send(ProjectileEvent::new(MustSync::Both, ProjectileStatsEvent::new(id, None, ProjectileStatsUpdate::Kill)));
                    projectiles.kill_ent(id);
                }
                else {
                    // println!("Projectile {id} DIDNT hit world with speed {:?}", pos.spd);
                    let static_type = &projectiles.static_types[projectiles.stats[id].get_id()];
                    let new_pos = pos.pos + pos.spd;
                    let mut new_spd = pos.spd;
                    new_spd += static_type.stats.gravity_felt;
                    projectiles.tunnels.pos_out.send(ProjectileEvent::new(MustSync::Both, PositionEvent::new(id, None, PositionUpdate::UpdatePosAndMoveSpeedAndHint(new_pos, new_spd, new_hint))));
                
                }
                // entity/world collisions, death on collision ? (need to handle world piercing case)
            }
        },
        EntityTurn::reclaim => if !reclaim.stats[id].dead {
            if reclaim.health[id].is_dead() {
                do_death_accounting(TID::reclaim(id), units, buildings, projectiles, reclaim);
                let static_stats = &reclaim.static_types[reclaim.stats[id].get_id()];
                reclaim.mesh_info[id].free_mesh(&static_stats.mesh_info, &extra_data.mantle);
                reclaim.tunnels.stats_out.send(ReclaimEvent::new(MustSync::Both, SimpleStatsEvent::new(id, None, SimpleStatsUpdate::Kill)));
                reclaim.kill_ent(id);
            }
            
        },
        EntityTurn::teams => ()
    }
}

impl TurboGameEngineTID {
    pub fn unwrap_unit(self) -> usize {
        match self {
            TurboGameEngineTID::units(unit) => unit,
            _ => panic!("Not a unit"),
        }
    }
    pub fn unwrap_building(self) -> usize {
        match self {
            TurboGameEngineTID::buildings(building) => building,
            _ => panic!("Not a building"),
        }
    }
}

#[derive(GameEngine, Clone)]
#[rendering_engine = "MantleHandler"]
#[tick_stages = 4]
#[do_multiplayer]
pub struct TurboGameEngine {
    units:Unit,
    buildings:Building,
    projectiles:Projectile,
    reclaim:Reclaim,
    #[not_rendered]
    teams:Team,
    world:GameMap<CoolVoxel, RTSGen>,
    #[extra_data]
    extra_data:ExtraData
}