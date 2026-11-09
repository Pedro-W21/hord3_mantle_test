use std::f32::consts::PI;

use crate::shared::targeting::{TargetEnvironment, TargetTeam};

pub const SETGRID_SIZE:usize = 6;
pub const BUILDING_SET:usize = 0;
pub const GROUND_SET:usize = 1;
pub const AIR_SET:usize = 2;
pub const NAVAL_SET:usize = 3;
pub const PROJECTILE_SET:usize = 4;
pub const RECLAIM_SET:usize = 5;
pub const TICKS_PER_SECOND:usize = 60;
pub const TICKS_PER_SECOND_F32:f32 = TICKS_PER_SECOND as f32;
pub const TURRET_ORIENT_TOLERANCE:f32 = 0.02 * PI;
pub const RESOURCE_SHARE_SCALE:i64 = 100;

pub const CHUNK_SIDE:usize = 16;



pub const ASSIST_REPAIR_TARGETS:[(TargetEnvironment, TargetTeam); 4] = [
    (TargetEnvironment::Building, TargetTeam::Same),
    (TargetEnvironment::Ground, TargetTeam::Same),
    (TargetEnvironment::Air, TargetTeam::Same),
    (TargetEnvironment::Naval, TargetTeam::Same)
];