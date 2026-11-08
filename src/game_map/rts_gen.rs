use hord3::horde::game_engine::multiplayer::MustSync;
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{game_engine::CoolVoxel, game_map::{Generator, VoxelLight}};

#[derive(Clone, Copy, PartialEq, ToBytes, FromBytes)]
pub struct RTSGen {
    pub t:u8
}

#[derive(Clone, Copy, PartialEq, ToBytes, FromBytes)]
pub enum RTSGenUpdate {
    TestUpdate
}

impl Generator<CoolVoxel> for RTSGen {
    const PROVIDES_COLLISION:bool = false;
    type GeneratorUpdate = RTSGenUpdate;
    fn apply_update(update:Self::GeneratorUpdate, world:&mut super::GameMap<CoolVoxel, Self>) {
        
    }
    fn full_collision(&self, pos:hord3::horde::geometry::vec3d::Vec3Df, speed_nudge:hord3::horde::geometry::vec3d::Vec3Df) -> Option<super::Collision<CoolVoxel>> {
        None
    }
    fn generate(&self, pos:super::WorldVoxelPos) -> CoolVoxel {
        CoolVoxel::new(0, 0, VoxelLight::zero_light(), 0, None)
    }
    fn get_chunks_to_hide(&self, camera:&hord3::horde::rendering::camera::Camera) -> Vec<super::WorldChunkPos> {
        vec![]
    }
    fn get_chunks_to_show(&self, camera:&hord3::horde::rendering::camera::Camera) -> Vec<super::WorldChunkPos> {
        vec![]
    }
    fn get_chunks_to_unmesh(&self, camera:&hord3::horde::rendering::camera::Camera) -> Vec<super::WorldChunkPos> {
        vec![]
    }
    fn must_sync(update:&Self::GeneratorUpdate) -> hord3::horde::game_engine::multiplayer::MustSync {
        MustSync::No
    }
    fn simple_collision(&self, pos:hord3::horde::geometry::vec3d::Vec3Df) -> bool {
        false
    }
}