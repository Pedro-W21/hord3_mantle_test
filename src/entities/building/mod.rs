use std::f32::consts::PI;

use entity_derive::Entity;

use crate::{shared::{colliders::ComplexCollider, death_accounting::DependSet, health::Health, mesh_info::MeshInfo, position::Position, simple_position::SimplePos, stats::Stats, turrets::{Turrets, TurretsStore}}, turbo_game_engine::TurboGameEngineTID};

use hord3::{defaults::default_rendering::{mantle::api::{CPUInstanceData, MantleHandler}, vectorinator::{VectorinatorWrite, meshes::{MeshID, MeshInstance}}}, horde::{game_engine::{entity::{ChosenID, Component, ComponentEvent, EVecStopsIn, EVecStopsOut, Entity, EntityID, EntityIDAllocator, EntityVec, MultiplayerEntity, NewEntity, Renderable, StaticComponent, StaticEntity}, multiplayer::{Identify, MustSync}, position::EntityPosition, static_type_id::HasStaticTypeID}, geometry::{rotation::{Orientation, Rotation}, vec3d::Vec3Df}, utils::ARW}};

pub mod static_buildings;

#[derive(Entity, Clone)]
pub struct Building {
    #[used_in_new]
    #[used_in_render]
    #[must_sync]
    #[position]
    pos:SimplePos,
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
    mesh_info:MeshInfo
}

impl<ID:Identify> RenderBuilding<MantleHandler, ID> for Building {
    fn do_render_changes(rendering_data: &mut MantleHandler, pos: &mut SimplePos, stats: &mut Stats, turrets: &mut Turrets, mesh_info: &mut MeshInfo, static_type: &StaticBuilding<ID>) {
        match mesh_info.main_instance_id {
            Some(id) => {
                rendering_data.update_instance_no_rep(static_type.mesh_info.mesh_id.clone(), id, CPUInstanceData {
                    position:pos.pos,
                    speed:Vec3Df::zero(),
                    rotation:Orientation::zero()
                });
            },
            None => {
                if rendering_data.get_mesh(static_type.mesh_info.mesh_id.clone()).is_none() {
                    rendering_data.set_or_add_mesh_no_rep(static_type.mesh_info.mesh_id.unwrap_name(),static_type.mesh_info.mesh_data.clone(), Vec::new());
                }
                else {
                    mesh_info.main_instance_id = Some(rendering_data.add_instance_no_rep(static_type.mesh_info.mesh_id.clone(), CPUInstanceData {
                        position:pos.pos,
                        speed:Vec3Df::zero(),
                        rotation:Orientation::zero()
                    }))
                }
            }
        }
        if mesh_info.turret_instances.len() == 0 {
            mesh_info.turret_instances = vec![None ; static_type.turrets.get_full_count()];
        }
        match &turrets.turrets {
            TurretsStore::SingleTurret(turret) => {
                let self_pos = pos.pos + static_type.turrets.turrets[0].positions[turret.static_type_pos as usize];
                match mesh_info.turret_instances[0] {
                    Some(id) => {
                        rendering_data.update_instance_no_rep(static_type.turrets.turrets[0].mesh_info.mesh_id.clone(), id, CPUInstanceData {
                            position:self_pos,
                            speed:Vec3Df::zero(),
                            rotation: Orientation::new(turret.yaw, turret.pitch - PI/2.0, 0.0)
                        });
                    },
                    None => {
                        if rendering_data.get_mesh(static_type.turrets.turrets[0].mesh_info.mesh_id.clone()).is_none() {
                            rendering_data.set_or_add_mesh_no_rep(static_type.turrets.turrets[0].mesh_info.mesh_id.unwrap_name(),static_type.turrets.turrets[0].mesh_info.mesh_data.clone(), Vec::new());
                        }
                        else {
                            mesh_info.turret_instances[0] = Some(rendering_data.add_instance_no_rep(static_type.turrets.turrets[0].mesh_info.mesh_id.clone(), CPUInstanceData {
                                position:self_pos,
                                speed:Vec3Df::zero(),
                                rotation: Orientation::new(turret.yaw, turret.pitch - PI/2.0, 0.0)
                            }))
                        }
                    }
                }
            },
            TurretsStore::MultiTurret(turrets) => for (i, turret) in turrets.iter().enumerate() {
                let static_turret_type = &static_type.turrets.turrets[turret.static_id];
                let self_pos = pos.pos + static_turret_type.positions[turret.static_type_pos as usize];
                match mesh_info.turret_instances[i] {
                    Some(id) => {
                        //dbg!(turret.pitch);
                        rendering_data.update_instance_no_rep(static_turret_type.mesh_info.mesh_id.clone(), id, CPUInstanceData {
                            position:self_pos,
                            speed:Vec3Df::zero(),
                            rotation: Orientation::new(turret.yaw, turret.pitch - PI/2.0, 0.0)
                        });
                    },
                    None => {
                        if rendering_data.get_mesh(static_turret_type.mesh_info.mesh_id.clone()).is_none() {
                            rendering_data.set_or_add_mesh_no_rep(static_turret_type.mesh_info.mesh_id.unwrap_name(),static_turret_type.mesh_info.mesh_data.clone(), Vec::new());
                        }
                        else {
                            mesh_info.turret_instances[i] = Some(rendering_data.add_instance_no_rep(static_turret_type.mesh_info.mesh_id.clone(), CPUInstanceData {
                                position:self_pos,
                                speed:Vec3Df::zero(),
                                rotation: Orientation::new(turret.yaw, turret.pitch - PI/2.0,0.0)
                            }))
                        }
                    }
                }
            },
        }
        
    }
}

impl<ID:Identify> NewEntity<Building, ID> for NewBuilding<ID> {
    fn get_ent(self, static_type:&<Building as Entity<ID>>::SE) -> Building {
        let pos = self.pos.pos;
        Building { pos: self.pos, stats: self.stats, mesh_info:MeshInfo::new(), collider: static_type.collider.get_moved(pos, Orientation::zero()), health: <Health as Component<ID>>::from_static(&static_type.health), depend: <DependSet<TurboGameEngineTID> as Component<ID>>::from_static(&static_type.depend), turrets:<Turrets as Component<ID>>::from_static(&static_type.turrets)  }
    }
} 