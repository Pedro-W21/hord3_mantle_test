use std::f32::consts::PI;

use entity_derive::Entity;

use crate::{shared::{actions::Actions, colliders::ComplexCollider, death_accounting::DependSet, health::Health, mesh_info::MeshInfo, planner::Planner, position::Position, stats::Stats, turrets::{Turrets, TurretsStore}}, turbo_game_engine::TurboGameEngineTID};

use hord3::{defaults::default_rendering::{mantle::api::{CPUInstanceData, MantleHandler}, vectorinator::{VectorinatorWrite, meshes::{MeshID, MeshInstance}}}, horde::{game_engine::{entity::{ChosenID, Component, ComponentEvent, EVecStopsIn, EVecStopsOut, Entity, EntityID, EntityIDAllocator, EntityVec, MultiplayerEntity, NewEntity, Renderable, StaticComponent, StaticEntity}, multiplayer::{Identify, MustSync}, position::EntityPosition, static_type_id::HasStaticTypeID}, geometry::{rotation::{Orientation, Rotation}, vec3d::Vec3Df}, utils::ARW}};


pub mod static_entities;

#[derive(Entity, Clone)]
pub struct Unit {
    #[used_in_new]
    #[used_in_render]
    #[must_sync]
    #[position]
    pos:Position,
    #[used_in_new]
    #[static_id]
    stats:Stats,
    collider:ComplexCollider,
    health:Health,
    depend:DependSet<TurboGameEngineTID>,
    #[used_in_render]
    turrets:Turrets,
    #[used_in_render]
    #[no_sync]
    mesh_info:MeshInfo,
    actions:Actions,
    planner:Planner
}

impl<ID:Identify> RenderUnit<MantleHandler, ID> for Unit {
    fn do_render_changes(rendering_data: &mut MantleHandler, pos: &mut Position, stats: &mut Stats, turrets: &mut Turrets, mesh_info: &mut MeshInfo, static_type: &StaticUnit<ID>) {
        match mesh_info.main_instance_id {
            Some(id) => {
                rendering_data.update_instance_no_rep(static_type.mesh_info.mesh_id.clone(), id, CPUInstanceData {
                    position:pos.pos,
                    speed:pos.spd,
                    rotation:pos.orientation
                });
            },
            None => {
                if rendering_data.get_mesh(static_type.mesh_info.mesh_id.clone()).is_none() {
                    rendering_data.set_or_add_mesh_no_rep(static_type.mesh_info.mesh_id.unwrap_name(),static_type.mesh_info.mesh_data.clone(), Vec::new());
                }
                else {
                    mesh_info.main_instance_id = Some(rendering_data.add_instance_no_rep(static_type.mesh_info.mesh_id.clone(), CPUInstanceData {
                        position:pos.pos,
                        speed:pos.spd,
                        rotation:pos.orientation
                    }))
                }
            }
        }
        if mesh_info.turret_instances.len() == 0 {
            mesh_info.turret_instances = vec![None ; static_type.turrets.get_full_count()];
        }
        match &turrets.turrets {
            TurretsStore::SingleTurret(turret) => {
                let static_turret_type = &static_type.turrets.turrets[turret.static_id];
                let self_pos = pos.pos + Rotation::from_orientation(pos.orientation).rotate(static_turret_type.positions[turret.static_type_pos as usize]);
                match mesh_info.turret_instances[0] {
                    Some(id) => {
                        //dbg!(turret.yaw, turret.pitch);
                        rendering_data.update_instance_no_rep(static_type.turrets.turrets[0].mesh_info.mesh_id.clone(), id, CPUInstanceData {
                            position:self_pos,
                            speed:pos.spd,
                            rotation:pos.orientation + Orientation::new(turret.yaw, turret.pitch - PI/2.0, 0.0)
                        });
                    },
                    None => {
                        if rendering_data.get_mesh(static_type.turrets.turrets[0].mesh_info.mesh_id.clone()).is_none() {
                            rendering_data.set_or_add_mesh_no_rep(static_type.turrets.turrets[0].mesh_info.mesh_id.unwrap_name(),static_type.turrets.turrets[0].mesh_info.mesh_data.clone(), Vec::new());
                        }
                        else {
                            mesh_info.turret_instances[0] = Some(rendering_data.add_instance_no_rep(static_type.turrets.turrets[0].mesh_info.mesh_id.clone(), CPUInstanceData {
                                position:self_pos,
                                speed:pos.spd,
                                rotation:pos.orientation + Orientation::new(turret.yaw, turret.pitch - PI/2.0, 0.0)
                            }))
                        }
                    }
                }
            },
            TurretsStore::MultiTurret(turrets) => for (i, turret) in turrets.iter().enumerate() {
                let static_turret_type = &static_type.turrets.turrets[turret.static_id];
                let self_pos = pos.pos + Rotation::from_orientation(pos.orientation).rotate(static_turret_type.positions[turret.static_type_pos as usize]);
                match mesh_info.turret_instances[i] {
                    Some(id) => {
                        rendering_data.update_instance_no_rep(static_turret_type.mesh_info.mesh_id.clone(), id, CPUInstanceData {
                            position:self_pos,
                            speed:pos.spd,
                            rotation:pos.orientation + Orientation::new(turret.yaw, turret.pitch - PI/2.0, 0.0)
                        });
                    },
                    None => {
                        if rendering_data.get_mesh(static_turret_type.mesh_info.mesh_id.clone()).is_none() {
                            rendering_data.set_or_add_mesh_no_rep(static_turret_type.mesh_info.mesh_id.unwrap_name(),static_turret_type.mesh_info.mesh_data.clone(), Vec::new());
                        }
                        else {
                            mesh_info.turret_instances[i] = Some(rendering_data.add_instance_no_rep(static_turret_type.mesh_info.mesh_id.clone(), CPUInstanceData {
                                position:self_pos,
                                speed:pos.spd,
                                rotation:pos.orientation + Orientation::new(turret.yaw, turret.pitch - PI/2.0, 0.0)
                            }))
                        }
                    }
                }
            },
        }
    }
}

impl<ID:Identify> NewEntity<Unit, ID> for NewUnit<ID> {
    fn get_ent(self, static_type:&<Unit as Entity<ID>>::SE) -> Unit {
        let pos = self.pos.pos;
        Unit { pos: self.pos, stats: self.stats, actions:<Actions as Component<ID>>::from_static(&static_type.actions), planner:<Planner as Component<ID>>::from_static(&static_type.planner), mesh_info:MeshInfo::new(), collider: static_type.collider.clone(), health: <Health as Component<ID>>::from_static(&static_type.health), depend: <DependSet<TurboGameEngineTID> as Component<ID>>::from_static(&static_type.depend), turrets:<Turrets as Component<ID>>::from_static(&static_type.turrets) }
    }
} 