#![feature(portable_simd)]
#![feature(int_roundings)]
#![feature(mpmc_channel)]
use std::{collections::HashMap, f32::consts::PI, path::PathBuf, simd::Simd, sync::{Arc, RwLock, atomic::{AtomicUsize, Ordering}, mpmc::{self, channel}}, thread, time::{Duration, Instant}};

use crate::{game_engine::CoolGameEngineTID, game_entity::colliders::AABB, game_map::rts_gen::RTSGen, tile_editor::nth_texture};
use cosmic_text::{Color, Font, Metrics};
use crate::game_3d_models::{grey_sphere_mesh, lit_selection_cube, simple_line, sphere_mesh, textured_sphere_mesh, xyz_mesh};
use crate::game_engine::{CoolGameEngineBase, CoolVoxel, CoolVoxelType, ExtraData};
use crate::game_entity::{Collider, GameEntityVec, Movement, NewGameEntity, StaticCollider, StaticGameEntity, StaticMeshInfo, StaticMovement, StaticStats, Stats};
use crate::game_input_handler::GameInputHandler;
use crate::game_map::{get_f64_pos, get_float_pos, light_spreader::{LightPos, LightSpread}, ChunkDims, GameMap, VoxelLight};
use crate::game_tasks::{GameTask, GameTaskTaskHandler, GameUserEvent};
use crate::gui_elements::{list_choice::get_list_choice, number_config::get_number_config};
use hord3::{defaults::{default_frontends::minifb_frontend::MiniFBWindow, default_rendering::mantle::{api::MantleHandler, meshes::MeshID}, default_ui::simple_ui::{SimpleUI, UIDimensions, UIElement, UIElementBackground, UIElementContent, UIElementID, UIEvent, UIUnit, UIUserAction, UIVector}}, horde::{frontend::{HordeWindowDimensions, WindowingHandler}, game_engine::{entity::Renderable, world::{WorldComputeHandler, WorldHandler}}, geometry::{plane::EquationPlane, rotation::{Orientation, Rotation}, vec3d::{Vec3D, Vec3Df}}, rendering::{camera::Camera, framebuffer::HordeColorFormat}, scheduler::{EmptyBackgroundTask, HordeScheduler, HordeTaskQueue, HordeTaskSequence, SequencedTask}, sound::{SoundRequest, WaveIdentification, WavePosition, WaveRequest, WaveSink, Waves}}};
use noise::{NoiseFn, Perlin, Seedable};
use crate::tile_editor::{get_tile_voxels, TileEditorData};

use crate::{game_entity::{actions::{Action, ActionKind, ActionSource, ActionTimer, ActionsEvent, ActionsUpdate, StaticGameActions}, director::{Director, DirectorKind, StaticDirector}, planner::StaticPlanner}, game_map::get_voxel_pos};


pub fn run_game(mantle_handler:MantleHandler) {
    let dims = ChunkDims::new(8, 8, 8);
    let mut world = GameMap::new(100, dims.clone(), get_tile_voxels(), (255,255,255), 1, RTSGen {t:0}, 5.0, 15);
    let mut perlin = Perlin::new(13095);
    let mut world_height = 15.0;
    let mut water_level = 10.0;
    let start = Vec3D::new(-30, -20, -2);
    let end = Vec3D::new(20, 20, 20);
    let mut ground_at = vec![0; ((end.x - start.x) * dims.chunk_length_i * (end.y - start.y) * dims.chunk_width_i) as usize];
    let length_f64 = ((end.x - start.x) * 8 ) as f64;
    world.generate_chunks(start, end, &mut |pos| {
        let pos_3D = (get_f64_pos(pos) * 0.01);
        let value_2D = (perlin.get([pos_3D.x, pos_3D.y]) + 1.0) * 0.5;
        let local_world_height = world_height - (((pos.x - start.x) * 8) as f64/length_f64) * world_height;
        let actual_height = local_world_height + world_height * value_2D * 2.0;
        if (pos.z as f64) < actual_height || (pos.z as f64) < water_level {
            let ground_pos = (pos.x - (start.x * dims.chunk_length_i) + (pos.y - (start.y * dims.chunk_width_i)) * ((end.y - start.y) * dims.chunk_width_i)) as usize;
            if (pos.z as f64) < water_level {
                if ground_at[ground_pos] < pos.z {
                    ground_at[ground_pos] = water_level as i32;
                }
                CoolVoxel {voxel_type:7, orient:0, light:VoxelLight::random_light(), thickness:0}
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
        for i in 0..3 {
            let (x,y) = (fastrand::i32((start.x * 8)..(end.x * 8)), fastrand::i32((start.y * 8)..(end.y * 8)));
            let light_source = LightPos::new(Vec3D::new(x, y, ground_at[(x - (start.x * 8) + (y - (start.y * 8)) * ((end.y - start.y) * 8)) as usize] + 1), VoxelLight::slightly_less_random_light());
            let total_light_spread = LightSpread::calc_max_spread(&world_clone, light_source).get_all_spread();
            for light_pos in total_light_spread {
                world_clone.get_voxel_at_mut(light_pos.pos()).unwrap().light = light_pos.value().merge_with_other(&world_clone.get_voxel_at(light_pos.pos()).unwrap().light);
            }
            println!("light {}/100 done !", i+1);
        }
    }
    
    let entity_vec = GameEntityVec::new(1000);
    {
        let mut writer = entity_vec.get_write();
        writer.new_sct(StaticGameEntity{planner:StaticPlanner{},director:StaticDirector {kind:DirectorKind::Nothing},actions:StaticGameActions {base_actions:Vec::with_capacity(8)},movement:StaticMovement{}, mesh_info:StaticMeshInfo{mesh_id:MeshID::Name("EntityMesh".to_string()),mesh_data:vec![simple_line(-Vec3D::all_ones()*0.5, Vec3D::all_ones()*0.5, nth_texture(2), (255,255,255))]}, stats:StaticStats{}, collider:StaticCollider{init_aabb:AABB::new(-Vec3D::all_ones()*0.5, Vec3D::all_ones()*0.5)}});

        writer.new_sct(StaticGameEntity{planner:StaticPlanner{},director:StaticDirector {kind:DirectorKind::Nothing},actions:StaticGameActions {base_actions:Vec::with_capacity(8)},movement:StaticMovement{}, mesh_info:StaticMeshInfo{mesh_id:MeshID::Name("GREY_MESH".to_string()),mesh_data:vec![grey_sphere_mesh()]}, stats:StaticStats{}, collider:StaticCollider{init_aabb:AABB::new(-Vec3D::all_ones()*0.5, Vec3D::all_ones()*0.5)}});



        for i in 0..100 {
            let pos = Vec3D::new((fastrand::f32() - 0.5) * 2.0 * 150.0, (fastrand::f32() - 0.5) * 2.0 * 150.0, 150.0);
            writer.new_ent(NewGameEntity::new(Movement{against_wall:false, touching_ground:false,pos:pos, speed:Vec3D::zero(), orient:Orientation::zero(), rotat:Rotation::from_orientation(Orientation::zero())}, Stats {static_type_id:1, health:0, damage:0, stamina:0, ground_speed:0.2, jump_height:1.0}, Collider{team:0, collider:AABB::new(pos - Vec3D::all_ones() * 0.5, pos + Vec3D::all_ones() * 0.5)}, Director::new_with_random_name(DirectorKind::LLM), None));
            //writer.new_ent(NewGameEntity::new(Movement{against_wall:false, touching_ground:false,pos:pos, speed:Vec3D::zero(), orient:Orientation::zero(), rotat:Rotation::from_orientation(Orientation::zero())}, Stats {static_type_id:1, health:0, damage:0, stamina:0, ground_speed:0.2, jump_height:1.0}, Collider{team:0, collider:AABB::new(pos - Vec3D::all_ones() * 0.5, pos + Vec3D::all_ones() * 0.5)}, Director::new_with_random_name(DirectorKind::LLM(LLMDirector::new_with_goals(test_goals[i].clone())))));
        }
    }
    
    let entity_vec_2 = GameEntityVec::new(1000);
    
    let (waves, waves_handler, stream) = Waves::new(Vec::new(), 10);
    let world_handler = WorldHandler::new(world);
    
    let engine = CoolGameEngineBase::new(entity_vec, entity_vec_2, world_handler.clone(), Arc::new(mantle_handler.clone()), ExtraData {tick: Arc::new(AtomicUsize::new(0)), waves:waves_handler.clone()});
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
    let handler = GameTaskTaskHandler::new(engine.clone(), mantle_handler.clone(), waves);
    
    let queue = HordeTaskQueue::new(vec![HordeTaskSequence::new(vec![

        SequencedTask::StartTask(GameTask::DoMantleStuff),
        SequencedTask::WaitFor(GameTask::DoMantleStuff),
        SequencedTask::StartTask(GameTask::PrepareRendering),
        SequencedTask::WaitFor(GameTask::PrepareRendering),
        SequencedTask::StartTask(GameTask::ApplyEvents),
        SequencedTask::StartTask(GameTask::UpdateSoundPositions),
        SequencedTask::WaitFor(GameTask::ApplyEvents),
        SequencedTask::StartTask(GameTask::Stage0),
        SequencedTask::WaitFor(GameTask::Stage0),
        SequencedTask::WaitFor(GameTask::UpdateSoundPositions),
        SequencedTask::StartTask(GameTask::UpdateSoundEverythingElse),
        SequencedTask::StartTask(GameTask::ApplyEvents),
        SequencedTask::WaitFor(GameTask::ApplyEvents),
        SequencedTask::StartTask(GameTask::Stage1),
        SequencedTask::WaitFor(GameTask::Stage1),
        SequencedTask::StartTask(GameTask::ApplyEvents),
        SequencedTask::WaitFor(GameTask::ApplyEvents),
        SequencedTask::WaitFor(GameTask::UpdateSoundEverythingElse),
        SequencedTask::StartTask(GameTask::DoMantleStuff),
        SequencedTask::WaitFor(GameTask::DoMantleStuff),
        ]
    )
    ], Vec::new());
    println!("Hello, world!");
    let mut scheduler:HordeScheduler<GameTask, EmptyBackgroundTask> = HordeScheduler::new(queue.clone(), handler, 16, 32);
    let mut input_handler = GameInputHandler::new(mouse2.clone(), 3.0, outside_events);
    // let mut tile_editor = TileEditorData::new(simpleui.clone(), input_handler.get_new_camera(), mouse2);
    {
        // tile_editor.initial_ui_work();
    }
    println!("FINISHED INITIAL");
    /*let mut sequence = CameraSequence::new(vec![
        CameraMovement::new(
            vec![
                CameraMovementElement::MoveFromToLinear { from: Vec3Df::zero(), to: Vec3Df::all_ones() * 180.0 },
                CameraMovementElement::MovementShake { ranges: Vec3Df::all_ones() * 0.005 },
                CameraMovementElement::PointAt { position: Vec3Df::new(-10.0, 30.0, 10.0) },
                CameraMovementElement::ConstantOrientChange { change: Orientation::new(-PI/4.0, 0.0, 0.0) }
            ],
            CameraMovementDuration::RealTime { duration: Duration::from_secs_f32(2.0) }
        ),
        CameraMovement::new(
            vec![
                CameraMovementElement::MoveFromToLinear { from: Vec3Df::all_ones() * 80.0, to: Vec3Df::all_ones() * 80.0 },
                CameraMovementElement::RotateFromToLinear { from: Orientation::zero(), to: Orientation::new(0.0, 2.0, 0.0) }
            ],
            CameraMovementDuration::RealTime { duration: Duration::from_secs_f32(2.0) }
        ),
    ]);*/
    // let mut cutscene = get_real_demo_cutscene(&viewport_data);
    let mut i = 0;
    loop {
        //println!("{i}");

        let mut start = Instant::now();
        input_handler.update_keyboard();
        let new_camera = {
            let new_camera =input_handler.get_new_camera();
            mantle_handler.update_camera_no_rep(new_camera.clone());//(i as f32 / 500.0) * PI/2.0));
            if i > 400 {
                let mut reader = engine.entity_1.get_read();

                let ent = fastrand::usize(0..reader.actions.len());
                let pos = reader.movement[ent].pos + Vec3D::new((fastrand::f32() - 0.5) * 2.0 * 50.0, (fastrand::f32() - 0.5) * 2.0 * 50.0, 0.0);
                let voxel_pos = get_voxel_pos(pos);
                let target_pos = engine.world.world.read().unwrap().get_ceiling_at(voxel_pos, 100) + Vec3D::new(0, 0, 1);
                let mut counter = reader.actions[ent].get_counter().clone();
                let next_action = counter.get_next_id();
                if reader.planner[ent].plans_len() < 1 {
                    reader.tunnels.actions_out.send(ActionsEvent::new(ent, None, ActionsUpdate::AddAction(Action::new(next_action, engine.extra_data.tick.load(Ordering::Relaxed), ActionTimer::Delay(500), ActionKind::PathToPosition(Vec3Df::new(target_pos.x as f32, target_pos.y as f32, target_pos.z as f32), 0.7), ActionSource::Director))));
                    reader.tunnels.actions_out.send(ActionsEvent::new(ent, None, ActionsUpdate::UpdateCounter(counter)));
                }
                
            }
            // dbg!(new_camera.clone());
            engine.extra_data.tick.fetch_add(1, Ordering::Relaxed);

            /*thread::sleep(Duration::from_millis(10));*/
            new_camera
        };
        {
            let first_ent = engine.entity_1.get_read();
            let second_ent = engine.entity_2.get_read();
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
