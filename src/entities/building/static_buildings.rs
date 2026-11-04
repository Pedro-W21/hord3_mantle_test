use hord3::horde::geometry::vec3d::Vec3Df;

use std::f32::consts::PI;

use hord3::{defaults::default_rendering::mantle::meshes::MeshID, horde::{game_engine::multiplayer::Identify}};

use crate::{entities::unit::StaticUnit, game_3d_models::{cannon_barrel, factory, grey_sphere_mesh}, shared::{actions::StaticActions, attack::{Attack, AttackType}, colliders::{AABB, BoundingCollider, BoundingSphere, ComplexCollider, SubCollider}, death_accounting::StaticDependSet, health::StaticHealth, mesh_info::StaticMeshInfo, planner::StaticPlanner, position::StaticPosition, resources::{ResourceCost, ResourceProduction, ResourceStorage}, stats::StaticStats, targeting::TargetEnvironment, turrets::{AllowedAction, AllowedActions, BuildableBuildings, BuildableUnits, StaticTurret, StaticTurrets}}};


use crate::{entities::building::StaticBuilding, shared::simple_position::StaticSimplePos};

pub fn get_test_attack_factory<ID:Identify>() -> StaticBuilding<ID> {
    StaticBuilding {
        pos: StaticSimplePos {
            a:0
        },
        stats: StaticStats {
            upgrade_speed:1.0,
            build_contribution_cost:ResourceCost::new(10, 10, 0),
            repair_contribution_cost:ResourceCost::new(10, 10, 0),
            assist_contribution_cost:ResourceCost::new(10, 10, 0),
            resource_storage:ResourceStorage::new(100_000, 10_000_000, 50),
            resource_production:ResourceProduction::new(1, 10),
            default_env:TargetEnvironment::Ground,
            construction_quantity:10000,
            building_footprint:AABB::new(Vec3Df::all_ones() * -1.0 + Vec3Df::new(0.0, 0.0, 1.0), Vec3Df::all_ones() * 1.0)
        },
        collider: ComplexCollider::new(
            AABB::new(Vec3Df::all_ones() * -1.0 + Vec3Df::new(0.0, 0.0, 1.0), Vec3Df::all_ones() * 1.0),
            vec![
                SubCollider::new(BoundingCollider::BS(BoundingSphere::new(Vec3Df::new(0.0, 0.0, 1.0), 1.5)), vec![])
            ]
        ),
        health: StaticHealth {
            starting_effects:vec![],
            base_max_health:1000
        },
        depend: StaticDependSet {
            size_hint:8
        },
        turrets: StaticTurrets {
            turrets: vec![
                StaticTurret {
                    positions:vec![
                        Vec3Df::new(1.0, 1.0, 2.5)
                    ],
                    allowed_actions:AllowedActions::ManyActions(vec![
                        AllowedAction::Attack(Attack {
                            range:(0.0, 100.0),
                            charge_ticks:Some(100),
                            cost:None,
                            cost_during_charge:None,
                            attack_type:AttackType::Projectile { id: 0 },
                            target_envs:vec![
                                TargetEnvironment::Ground,
                                TargetEnvironment::Building,
                                TargetEnvironment::Air,
                                TargetEnvironment::Naval
                            ],
                        }),
                    ]),
                    yaw_range: (0.0, PI),
                    pitch_range: (0.0, PI),
                    max_turn_rate: PI/24.0,
                    max_elevation_rate:PI/36.0,
                    max_health:1.0,
                    independent:false,
                    must_turn:true,
                    mesh_info:StaticMeshInfo{
                        mesh_id:MeshID::Name("CANNON_BARREL".to_string()),
                        mesh_data:vec![cannon_barrel()]
                    },
                    resource_share_cutoff:1
                },
                StaticTurret {
                    positions:vec![
                        Vec3Df::new(0.0, 2.0, 1.5)
                    ],
                    allowed_actions:AllowedActions::ManyActions(vec![
                        AllowedAction::BuildUnit { units: BuildableUnits::Specific(vec![0]), build_rate: 10, range: 10.0, placement: Vec3Df::new(0.0, 2.0, 0.5) }
                    ]),
                    yaw_range: (0.0, PI),
                    pitch_range: (0.0, PI),
                    max_turn_rate: PI/24.0,
                    max_elevation_rate:PI/36.0,
                    max_health:1.0,
                    independent:false,
                    must_turn:true,
                    mesh_info:StaticMeshInfo{
                        mesh_id:MeshID::Name("CANNON_BARREL".to_string()),
                        mesh_data:vec![cannon_barrel()]
                    },
                    resource_share_cutoff:1
                },
            ]
        },
        mesh_info: StaticMeshInfo{
            mesh_id:MeshID::Name("FACTORY".to_string()),
            mesh_data:vec![factory()]
        },
    }
}