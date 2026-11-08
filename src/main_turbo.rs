#![feature(portable_simd)]
#![feature(int_roundings)]
#![feature(mpmc_channel)]
use std::{collections::HashMap, f32::consts::PI, path::PathBuf, simd::Simd, sync::{Arc, RwLock, atomic::{AtomicUsize, Ordering}, mpmc::{self, channel}}, thread, time::{Duration, Instant}};

use cosmic_text::{Color, Font, Metrics};
use crate::{entities::{building::{BuildingEvent, BuildingVec, static_buildings::get_test_attack_factory}, projectile::{ProjectileVec, static_projectiles::test_projectile}, reclaim::{ReclaimVec, simple_stats::SimpleStats}, team::{NewTeam, TeamVec, static_teams::get_default_team}, unit::{NewUnit, UnitEvent, UnitVec, static_entities::{get_test_engineer, get_test_tank}}}, game_3d_models::{grey_sphere_mesh, lit_selection_cube, simple_line, sphere_mesh, textured_sphere_mesh, xyz_mesh}, game_map::{get_voxel_pos, rts_gen::RTSGen}, shared::{actions::{Action, ActionKind, ActionSource, ActionTimer, ActionsEvent, ActionsUpdate, TID}, colliders::AABB, consts::CHUNK_SIDE, position::Position, stats::Stats, targeting::TargetEnvironment, turrets::{TurretAction, TurretActionKind, TurretsEvent}}, tile_editor::nth_texture, turbo_game_engine::{ExtraData, TurboGameEngineBase}, turbo_game_tasks::{TurboGameTask, TurboGameTaskTaskHandler}};
use crate::game_engine::{CoolVoxel, CoolVoxelType};
use crate::game_input_handler::GameInputHandler;
use crate::game_map::{get_f64_pos, get_float_pos, light_spreader::{LightPos, LightSpread}, ChunkDims, GameMap, VoxelLight};
use crate::game_tasks::{GameTask, GameTaskTaskHandler, GameUserEvent};
use crate::gui_elements::{list_choice::get_list_choice, number_config::get_number_config};
use hord3::{defaults::{default_frontends::minifb_frontend::MiniFBWindow, default_rendering::mantle::{api::MantleHandler, meshes::MeshID}, default_ui::simple_ui::{SimpleUI, UIDimensions, UIElement, UIElementBackground, UIElementContent, UIElementID, UIEvent, UIUnit, UIUserAction, UIVector}}, horde::{frontend::{HordeWindowDimensions, WindowingHandler}, game_engine::{entity::Renderable, multiplayer::{HordeMultiModeChoice, MustSync}, static_type_id::HasStaticTypeID, world::{WorldComputeHandler, WorldHandler}}, geometry::{plane::EquationPlane, rotation::{Orientation, Rotation}, vec3d::{Vec3D, Vec3Df}}, rendering::{camera::Camera, framebuffer::HordeColorFormat}, scheduler::{EmptyBackgroundTask, HordeScheduler, HordeTaskQueue, HordeTaskSequence, SequencedTask}, sound::{SoundRequest, WaveIdentification, WavePosition, WaveRequest, WaveSink, Waves}}};
use noise::{NoiseFn, Perlin, Seedable};
use crate::tile_editor::{get_tile_voxels, TileEditorData};


pub fn turbo_run_game(mantle_handler:MantleHandler) {
    let dims = ChunkDims::new(CHUNK_SIDE, CHUNK_SIDE, CHUNK_SIDE);
    let mut world = GameMap::new(10000, dims.clone(), get_tile_voxels(), (255,255,255), 1, RTSGen {t:0}, 16.0, 20);
    let mut perlin = Perlin::new(13095);
    let mut world_height = 15.0;
    let mut water_level = 10.0;
    let start = Vec3D::new(-240 / (CHUNK_SIDE as i32), -160 / (CHUNK_SIDE as i32), -16  / (CHUNK_SIDE as i32));
    let end = Vec3D::new(160  / (CHUNK_SIDE as i32), 160  / (CHUNK_SIDE as i32), 160 /  (CHUNK_SIDE as i32));
    let mut ground_at = vec![0; ((end.x - start.x) * dims.chunk_length_i * (end.y - start.y) * dims.chunk_width_i) as usize];
    let length_f64 = ((end.x - start.x) * CHUNK_SIDE as i32 ) as f64;
    world.generate_chunks(start, end, &mut |pos| {
        let pos_3D = (get_f64_pos(pos) * 0.01);
        let value_2D = (perlin.get([pos_3D.x, pos_3D.y]) + 1.0) * 0.5;
        let local_world_height = world_height - (((pos.x - start.x) * CHUNK_SIDE as i32) as f64/length_f64) * world_height;
        let actual_height = local_world_height + world_height * value_2D * 2.0;
        if (pos.z as f64) < actual_height || (pos.z as f64) < water_level {
            let ground_pos = (pos.x - (start.x * dims.chunk_length_i) + (pos.y - (start.y * dims.chunk_width_i)) * ((end.y - start.y) * dims.chunk_width_i)) as usize;
            if (pos.z as f64) < water_level {
                if ground_at[ground_pos] < pos.z {
                    ground_at[ground_pos] = water_level as i32;
                }
                if (pos.z as f64) == water_level - 1.0 && actual_height < water_level {
                    CoolVoxel {voxel_type:7, orient:0, light:VoxelLight::random_light(), thickness:fastrand::u8(30..220)}
                }
                else {
                    CoolVoxel {voxel_type:7, orient:0, light:VoxelLight::random_light(), thickness:0}
                }
            }
            else {
                if ground_at[ground_pos] < pos.z {
                    ground_at[ground_pos] = pos.z;
                }
                CoolVoxel {voxel_type:1 + ((actual_height - water_level)/(6.0*world_height * (1.0/6.0))).clamp(0.0, 5.99) as u16, orient:0, light:VoxelLight::random_light(), thickness:0}
            }
        } else {
            CoolVoxel {voxel_type:0, orient:0, light:VoxelLight::zero_light(), thickness:0}
        }
    }
    );
    let mut world_clone = world.clone();
    let mut spare_world = world.clone();
    {
        world_clone.change_mesh_vec(10);
        world_clone.set_min_light_levels((50,50,50));
        /*for i in 0..3 {
            let (x,y) = (fastrand::i32((start.x * 8)..(end.x * 8)), fastrand::i32((start.y * 8)..(end.y * 8)));
            let light_source = LightPos::new(Vec3D::new(x, y, ground_at[(x - (start.x * 8) + (y - (start.y * 8)) * ((end.y - start.y) * 8)) as usize] + 1), VoxelLight::slightly_less_random_light());
            let total_light_spread = LightSpread::calc_max_spread(&world_clone, light_source).get_all_spread();
            for light_pos in total_light_spread {
                world_clone.get_voxel_at_mut(light_pos.pos()).unwrap().light = light_pos.value().merge_with_other(&world_clone.get_voxel_at(light_pos.pos()).unwrap().light);
            }
            println!("light {}/100 done !", i+1);
        }*/
    }
    
    let entity_vec = UnitVec::new(1000);
    {
        let mut writer = entity_vec.get_write();
        writer.new_sct(get_test_tank());
        writer.new_sct(get_test_engineer());

        const SPAWN_RADIUS:f32 = 20.0;

        for i in 0..75 {
            let pos = Vec3D::new((fastrand::f32() - 0.5) * 2.0 * SPAWN_RADIUS, (fastrand::f32() - 0.5) * 2.0 * SPAWN_RADIUS, 50.0) + Vec3Df::new(80.0, -50.0, 0.0);
            writer.new_ent(NewUnit::new(Position::new().with_pos(pos), Stats::new(0, 0, TargetEnvironment::Ground, 0), MustSync::Both, None, None));
            //writer.new_ent(NewGameEntity::new(Movement{against_wall:false, touching_ground:false,pos:pos, speed:Vec3D::zero(), orient:Orientation::zero(), rotat:Rotation::from_orientation(Orientation::zero())}, Stats {static_type_id:1, health:0, damage:0, stamina:0, ground_speed:0.2, jump_height:1.0}, Collider{team:0, collider:AABB::new(pos - Vec3D::all_ones() * 0.5, pos + Vec3D::all_ones() * 0.5)}, Director::new_with_random_name(DirectorKind::LLM(LLMDirector::new_with_goals(test_goals[i].clone())))));
        }
        for i in 0..25 {
            let pos = Vec3D::new((fastrand::f32() - 0.5) * 2.0 * SPAWN_RADIUS, (fastrand::f32() - 0.5) * 2.0 * SPAWN_RADIUS, 50.0) + Vec3Df::new(80.0, -50.0, 0.0);
            writer.new_ent(NewUnit::new(Position::new().with_pos(pos), Stats::new(0, 1, TargetEnvironment::Ground, 0), MustSync::Both, None, None));
            //writer.new_ent(NewGameEntity::new(Movement{against_wall:false, touching_ground:false,pos:pos, speed:Vec3D::zero(), orient:Orientation::zero(), rotat:Rotation::from_orientation(Orientation::zero())}, Stats {static_type_id:1, health:0, damage:0, stamina:0, ground_speed:0.2, jump_height:1.0}, Collider{team:0, collider:AABB::new(pos - Vec3D::all_ones() * 0.5, pos + Vec3D::all_ones() * 0.5)}, Director::new_with_random_name(DirectorKind::LLM(LLMDirector::new_with_goals(test_goals[i].clone())))));
        }

        for i in 0..75 {
            let pos = Vec3D::new((fastrand::f32() - 0.5) * 2.0 * SPAWN_RADIUS, (fastrand::f32() - 0.5) * 2.0 * SPAWN_RADIUS, 50.0) + Vec3Df::new(80.0, 50.0, 0.0);
            writer.new_ent(NewUnit::new(Position::new().with_pos(pos), Stats::new(1, 0, TargetEnvironment::Ground, 0), MustSync::Both, None, None));
            //writer.new_ent(NewGameEntity::new(Movement{against_wall:false, touching_ground:false,pos:pos, speed:Vec3D::zero(), orient:Orientation::zero(), rotat:Rotation::from_orientation(Orientation::zero())}, Stats {static_type_id:1, health:0, damage:0, stamina:0, ground_speed:0.2, jump_height:1.0}, Collider{team:0, collider:AABB::new(pos - Vec3D::all_ones() * 0.5, pos + Vec3D::all_ones() * 0.5)}, Director::new_with_random_name(DirectorKind::LLM(LLMDirector::new_with_goals(test_goals[i].clone())))));
        }
        for i in 0..25 {
            let pos = Vec3D::new((fastrand::f32() - 0.5) * 2.0 * SPAWN_RADIUS, (fastrand::f32() - 0.5) * 2.0 * SPAWN_RADIUS, 50.0) + Vec3Df::new(80.0, 50.0, 0.0);
            writer.new_ent(NewUnit::new(Position::new().with_pos(pos), Stats::new(1, 1, TargetEnvironment::Ground, 0), MustSync::Both, None, None));
            //writer.new_ent(NewGameEntity::new(Movement{against_wall:false, touching_ground:false,pos:pos, speed:Vec3D::zero(), orient:Orientation::zero(), rotat:Rotation::from_orientation(Orientation::zero())}, Stats {static_type_id:1, health:0, damage:0, stamina:0, ground_speed:0.2, jump_height:1.0}, Collider{team:0, collider:AABB::new(pos - Vec3D::all_ones() * 0.5, pos + Vec3D::all_ones() * 0.5)}, Director::new_with_random_name(DirectorKind::LLM(LLMDirector::new_with_goals(test_goals[i].clone())))));
        }
    }
    
    let entity_vec_2 = BuildingVec::new(1000);

    {
        let mut writer = entity_vec_2.get_write();
        writer.new_sct(get_test_attack_factory());
    }
    
    let entity_vec_3 = ProjectileVec::new(1000);

    {
        let mut writer = entity_vec_3.get_write();
        writer.new_sct(test_projectile());
    }
    
    let entity_vec_4 = ReclaimVec::new(1000);
    
    let entity_vec_5 = TeamVec::new(1000);

    {
        let mut writer = entity_vec_5.get_write();
        writer.new_sct(get_default_team());
        for i in 0..5 {
            let mut stats = SimpleStats::new(i, 0);
            writer.new_ent(NewTeam::new(stats, MustSync::Both, None, None));
        }
    }
    
    let (waves, waves_handler, stream) = Waves::new(Vec::new(), 10);
    let world_handler = WorldHandler::new(world);
    
    let engine = TurboGameEngineBase::new(entity_vec, entity_vec_2, entity_vec_3, entity_vec_4, entity_vec_5, world_handler.clone(), Arc::new(mantle_handler.clone()), HordeMultiModeChoice::Client { adress: None, name: "a".to_string(), chat: channel().1 }, ExtraData {mantle:mantle_handler.clone(),tick: Arc::new(AtomicUsize::new(0)), waves:waves_handler.clone()});
    waves_handler.send_gec(engine.clone());
    let mouse = mantle_handler.get_mouse_state();
    let mouse2 = mantle_handler.get_mouse_state();

    let outside_events = mantle_handler.get_outside_events();
    // let (mut simpleui, user_events) = SimpleUI::new(20, 20, framebuf.clone(), mouse, channel().1);

    // TRES IMPORTANTTTTTTTTTTTTT
    //simpleui.add_many_connected_elements(get_list_choice(vec!["TerrainModifier".to_string(), "TileChooser".to_string(), "TerrainZoneModifier".to_string(), "LightSpreader".to_string()], UIVector::new(UIUnit::ParentWidthProportion(0.9), UIUnit::ParentHeightProportion(0.3)), UIDimensions::Decided(UIVector::new(UIUnit::ParentWidthProportion(0.1), UIUnit::ParentHeightProportion(0.3))), "Tools".to_string(), "rien".to_string()));
    
    {
        println!("START TEXTURE");
        for vox_type in get_tile_voxels() {
            mantle_handler.create_or_update_texture_no_rep(vox_type.texture_path.unwrap_or(String::from("textures/arbre.png")).as_str(), vox_type.name);
        }
        
        println!("DONE TEXTURE");
    }
    let handler = TurboGameTaskTaskHandler::new(engine.clone(), mantle_handler.clone(), waves);
    
    let queue = HordeTaskQueue::new(vec![HordeTaskSequence::new(vec![

        SequencedTask::StartTask(TurboGameTask::DoMantleStuff),
        SequencedTask::WaitFor(TurboGameTask::DoMantleStuff),
        SequencedTask::StartTask(TurboGameTask::PrepareRendering),
        SequencedTask::WaitFor(TurboGameTask::PrepareRendering),
        SequencedTask::StartTask(TurboGameTask::ApplyEvents),
        SequencedTask::StartTask(TurboGameTask::UpdateSoundPositions),
        SequencedTask::WaitFor(TurboGameTask::ApplyEvents),
        SequencedTask::StartTask(TurboGameTask::Stage0),
        SequencedTask::WaitFor(TurboGameTask::Stage0),
        SequencedTask::WaitFor(TurboGameTask::UpdateSoundPositions),
        SequencedTask::StartTask(TurboGameTask::UpdateSoundEverythingElse),
        SequencedTask::StartTask(TurboGameTask::ApplyEvents),
        SequencedTask::WaitFor(TurboGameTask::ApplyEvents),
        SequencedTask::StartTask(TurboGameTask::Stage1),
        SequencedTask::WaitFor(TurboGameTask::Stage1),
        SequencedTask::StartTask(TurboGameTask::ApplyEvents),
        SequencedTask::WaitFor(TurboGameTask::ApplyEvents),
        SequencedTask::StartTask(TurboGameTask::Stage2),
        SequencedTask::WaitFor(TurboGameTask::Stage2),
        SequencedTask::StartTask(TurboGameTask::ApplyEvents),
        SequencedTask::WaitFor(TurboGameTask::ApplyEvents),
        SequencedTask::StartTask(TurboGameTask::Stage3),
        SequencedTask::WaitFor(TurboGameTask::Stage3),
        SequencedTask::StartTask(TurboGameTask::ApplyEvents),
        SequencedTask::WaitFor(TurboGameTask::ApplyEvents),
        SequencedTask::StartTask(TurboGameTask::MultiPartOne),
        SequencedTask::WaitFor(TurboGameTask::MultiPartOne),
        SequencedTask::StartTask(TurboGameTask::MultiPartTwo),
        SequencedTask::WaitFor(TurboGameTask::MultiPartTwo),
        SequencedTask::StartTask(TurboGameTask::MultiPartThree),
        SequencedTask::WaitFor(TurboGameTask::MultiPartThree),
        SequencedTask::WaitFor(TurboGameTask::UpdateSoundEverythingElse),
        SequencedTask::StartTask(TurboGameTask::DoMantleStuff),
        SequencedTask::WaitFor(TurboGameTask::DoMantleStuff),
        ]
    )
    ], Vec::new());
    println!("Hello, world!");
    let mut scheduler:HordeScheduler<TurboGameTask, EmptyBackgroundTask> = HordeScheduler::new(queue.clone(), handler, 16, 32);
    let mut input_handler = GameInputHandler::new(mouse2.clone(), 3.0, outside_events);
    // let mut tile_editor = TileEditorData::new(simpleui.clone(), input_handler.get_new_camera(), mouse2);
    {
        // tile_editor.initial_ui_work();
    }
    println!("FINISHED INITIAL");
    let mut i = 0;
    let mut buildings_counter = 0;
    loop {
        let mut start = Instant::now();
        input_handler.update_keyboard();
        let new_camera = {
            let new_camera =input_handler.get_new_camera();
            mantle_handler.update_camera_no_rep(new_camera.clone());//(i as f32 / 500.0) * PI/2.0));
            if i > 400 {
                let mut reader = engine.units.get_read();
                let mut reader2 = engine.buildings.get_read();
                let mut reader3 = engine.projectiles.get_read();

                let ent = fastrand::usize(0..reader.actions.len());
                let pos = reader.pos[ent].pos + Vec3D::new((fastrand::f32() - 0.5) * 2.0 * 50.0, (fastrand::f32() - 0.5) * 2.0 * 50.0, 0.0);
                let voxel_pos = get_voxel_pos(pos);
                let stats = &reader.stats[ent];
                let target_pos = engine.world.world.read().unwrap().get_ceiling_at(voxel_pos, 100) + Vec3D::new(0, 0, 1);
                let mut counter = reader.actions[ent].get_counter().clone();
                let next_action = counter.get_next_id();
                if stats.get_id() == 1 && buildings_counter <= 2000 {
                    buildings_counter += 1;
                    let new_target = Vec3Df::new(target_pos.x as f32, target_pos.y as f32, target_pos.z as f32);
                    reader.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::<TID>::new(ent, None, ActionsUpdate::AddAction(Action::new(next_action, engine.extra_data.tick.load(Ordering::Relaxed), ActionTimer::Delay(500), ActionKind::BuildBuilding { position: new_target, building_type: 0, with_turret: None }, ActionSource::Director)))));
                    reader.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::<TID>::new(ent, None, ActionsUpdate::UpdateCounter(counter))));
                }
                else if reader.planner[ent].plans_len() < 1 && stats.get_id() == 0 {
                    reader.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::<TID>::new(ent, None, ActionsUpdate::AddAction(Action::new(next_action, engine.extra_data.tick.load(Ordering::Relaxed), ActionTimer::Delay(500), ActionKind::PathToPosition(Vec3Df::new(target_pos.x as f32, target_pos.y as f32, target_pos.z as f32), 0.7), ActionSource::Director)))));
                    reader.tunnels.actions_out.send(UnitEvent::new(MustSync::Both, ActionsEvent::<TID>::new(ent, None, ActionsUpdate::UpdateCounter(counter))));
                }


                println!("total units capacity : {} ; total buildings capacity : {} ; total projectiles capacity : {}", reader.actions.len(), reader2.collider.len(), reader3.collider.len());

                if reader2.collider.len() > 0 {
                    let build = fastrand::usize(0..reader2.turrets.len());
                    if reader.actions.len() < 12000 && reader2.turrets[build].turrets.unwrap_multi_ref()[1].action_slot.is_none() {
                        reader2.tunnels.turrets_out.send(BuildingEvent::new(MustSync::Both, TurretsEvent::new(build, None, crate::shared::turrets::TurretsUpdate::AddMultiAction(vec![None, Some(TurretAction::new(TurretActionKind::BuildUnit { unit_type: 0, progress: 0 }, 0))]))));
                    }
                }
                
            }
            // dbg!(new_camera.clone());
            engine.extra_data.tick.fetch_add(1, Ordering::Relaxed);

            /*thread::sleep(Duration::from_millis(10));*/
            new_camera
        };
        {
            let first_ent = engine.units.get_read();
            let second_ent = engine.buildings.get_read();
            let world = WorldComputeHandler::from_world_handler(&engine.world);
        }
        
        
        /*tile_editor.cam = new_camera;
        match user_events.try_recv() {
            Ok(evt) => {
                tile_editor.handle_user_event(evt);
            }
            Err(_) => ()
        }
        tile_editor.do_mouse_handling(&mut world_handler.world.write().unwrap());
        tile_editor.handle_keyboard(&input_handler, &mut world_handler.world.write().unwrap());
        tile_editor.do_rendering(&mantle_handler, &world_handler.world.read().unwrap());*/
        scheduler.initialise(queue.clone());
        scheduler.tick();
        let mut fps = 1.0/Instant::now().checked_duration_since(start).unwrap().as_secs_f64();
        println!("FPS : {}", fps);
        if fps >= 78.0 {
            thread::sleep(Duration::from_secs_f64(1.0/75.0 - Instant::now().checked_duration_since(start).unwrap().as_secs_f64()));
        }
        i += 1;
    }
    scheduler.end_threads();
}
