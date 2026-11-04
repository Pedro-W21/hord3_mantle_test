use hord3::{defaults::default_rendering::mantle::meshes::MeshID, horde::geometry::vec3d::Vec3Df};

use crate::{entities::projectile::{StaticProjectile, projectile_stats::{ProjectileAOE, ProjectileDeath, StaticProjectileStats}}, game_3d_models::simple_projectile, shared::{actions::TID, attack::Damage, colliders::{AABB, BoundingCollider, BoundingSphere, ComplexCollider, SubCollider}, death_accounting::StaticDependSet, position::StaticPosition, simple_health::StaticSimpleHealth, simple_mesh_info::StaticSimpleMeshInfo, targeting::TargetEnvironment}, turbo_game_engine::GRAVITY};


pub fn test_projectile() -> StaticProjectile<TID> {
    StaticProjectile {
        pos: StaticPosition {
            ground_speed:1.0,
            jump_height: 1.0,
            static_entity:true
        },
        stats: StaticProjectileStats {
            upgrade_speed:1.0,
            on_death:ProjectileDeath::Simple,
            aoe:ProjectileAOE::DirectDamage { range: 1.0, damage: Damage::Direct(200) },
            gravity_felt:Vec3Df::new(0.0, 0.0, -GRAVITY * 0.02),
            max_speed:10.0,
            target_environments:vec![
                TargetEnvironment::Ground,
                TargetEnvironment::Air,
                TargetEnvironment::Naval,
                TargetEnvironment::Building,
            ],
            ttl:1000
        },
        collider: ComplexCollider::new(AABB::new(Vec3Df::all_ones() * -0.25, Vec3Df::all_ones() * 0.25), vec![
                SubCollider::new(BoundingCollider::BS(BoundingSphere::new(Vec3Df::zero(), 0.25)), vec![])
            ]),
        health: StaticSimpleHealth {base_max_health:100},
        depend: StaticDependSet {size_hint:2},
        mesh_info: StaticSimpleMeshInfo {
            mesh_id:MeshID::Name("TEST_PROJECTILE".to_string()),
            mesh_data:vec![simple_projectile()]
        }
    }
}