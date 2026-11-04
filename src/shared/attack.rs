use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::shared::{actions::{AttackTarget, TID}, health::StatusEffect, resources::ResourceCost, targeting::TargetEnvironment};

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub enum Damage {
    Direct(i32),
    Complex {damage:i32, effects:Vec<StatusEffect>}
}

impl Damage {
    pub fn get_simple_damage(&self) -> i32 {
        match self {
            Self::Direct(val) => *val,
            Self::Complex { damage, effects } => *damage
        }
    }
}

#[derive(Clone, Debug)]
pub struct Attack {
    /// min, max
    pub range:(f32,f32),
    pub charge_ticks:Option<usize>,
    pub cost:Option<ResourceCost>,
    pub cost_during_charge:Option<ResourceCost>,
    pub attack_type:AttackType,
    pub target_envs:Vec<TargetEnvironment>,

}


pub type ProjectileStaticID = usize;

#[derive(Clone, Debug)]
pub enum AttackType {
    Hitscan {
        damage:Damage,
        pierce:PierceKind,
        beam_width:f32,
        end_of_beam:Option<ProjectileStaticID>,
        continuous:bool,
    },
    Projectile {
        id:ProjectileStaticID,
    },
    ManyProjectiles {
        ids:Vec<ProjectileStaticID>
    },
}

#[derive(Clone, Debug)]
pub enum PierceKind {
    No,
    Enemies(u32),
    Terrain(u32),
    EnemiesAndTerrain{enemies:u32, terrain:u32},
    OnlyHitTarget,
    AlwaysHitTarget,
}

#[derive(Clone, Debug, PartialEq, FromBytes, ToBytes)]
pub struct AttackCast {
    pub current_charge:usize,
    pub target:AttackTarget,

}