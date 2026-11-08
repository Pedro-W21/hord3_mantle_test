use std::{collections::{HashMap, VecDeque}, f32::consts::{PI, SQRT_2}, fmt::Debug, simd::{Simd, num::SimdFloat}, sync::{Arc, LazyLock}};

use hord3::{defaults::default_rendering::{mantle::{api::{ApiLod, CPUInstanceData, CPUVertexData, MantleHandler, TrianglePoint}, meshes::MeshID}, vectorinator_binned::triangles::{collux_f32_a_u8, collux_one_simd_to_u8_level, collux_u8_a_f32, collux_u8_tuple_to_f32_simd}}, horde::{game_engine::{entity::Renderable, multiplayer::{Identify, MustSync}, world::{World, WorldEvent}}, geometry::{rotation::{Orientation, Rotation}, vec3d::{Vec3D, Vec3Df}}, rendering::{RenderingBackend, camera::Camera}}, tests::engine_derive_test::TestRB};
use to_from_bytes::{FromBytes, ToBytes};
use to_from_bytes_derive::{FromBytes, ToBytes};
use vec_sparse_grid::{SetGrid, SetGridUpdate};

use crate::{game_map::vec_sparse_grid::{TurboGrid, VecGrid}, shared::consts::SETGRID_SIZE, tile_editor::get_world_textures_list};


pub mod light_spreader;
pub mod raycaster;
pub mod sparse_grid;
pub mod vec_sparse_grid;
pub mod rts_gen;
pub mod turbo_set;

pub const VEC_LENGTH:usize = 4;
pub const SET_CAPACITY:usize = 16;


#[derive(Clone, Copy,ToBytes,FromBytes, PartialEq, Eq, Hash, Debug)]
pub struct VoxelLight {
    level:u8,
    r:u8,
    g:u8,
    b:u8
}

impl VoxelLight {
    pub fn new(level:u8, r:u8, g:u8, b:u8) -> Self {
        Self { level, r, g, b }
    }
    pub fn random_light() -> Self {
        Self::new(fastrand::u8(0..255), fastrand::u8(0..255), fastrand::u8(0..255), fastrand::u8(0..255))
    }
    pub fn zero_light() -> Self {
        Self { level: 0, r: 0, g: 0, b: 0 }
    }
    pub fn max_light() -> Self {
        Self { level:255, r:255, g:255, b:255 }
    }
    pub fn slightly_less_random_light() -> Self {
        let mut rgb = [0 ; 3];
        for i in 0..fastrand::usize(1..3) {
            rgb[fastrand::usize(0..3)] = fastrand::u8(200..=255);
        }
        Self { level: fastrand::u8(230..=255), r: rgb[0], g: rgb[1], b: rgb[2] }
    }
    pub fn merge_with_other(&self, other:&Self) -> Self {
        let collux_self = collux_u8_tuple_to_f32_simd((self.r, self.g, self.b), self.level);
        let collux_other = collux_u8_tuple_to_f32_simd((other.r, other.g, other.b), other.level);
        let level_self = Simd::splat(collux_self.to_array()[3]);
        let level_other = Simd::splat(collux_other.to_array()[3]);
        let a01 = ((Simd::splat(1.0) - level_self) * level_other + level_self).simd_max(Simd::splat(0.001));
        let new_col = (((Simd::splat(1.0) - level_self) * level_other * collux_other + level_self * collux_self)/a01).simd_min(Simd::splat(1.0));
        let ((r,g,b), _) = collux_one_simd_to_u8_level(new_col);
        /* 
        let total_per_color = (collux_self + collux_other).simd_max(Simd::splat(0.01));
        let part_per_color_self = collux_self/total_per_color;
        let part_per_color_other = collux_other/total_per_color;
        let level = collux_one_simd_to_u8_level(collux_self).1.max(collux_one_simd_to_u8_level(collux_other).1);
        let ((r,g,b), nlevel) = collux_one_simd_to_u8_level(collux_self * part_per_color_self + collux_other * part_per_color_other);

        let ((nr,ng,nb), nlevel) = collux_one_simd_to_u8_level(total_per_color);
        let level_self = self.level as f32 / 255.0;
        let level_other = other.level as f32 / 255.0;
        let total_level = level_self + level_other;
        let part_of_self = level_self/total_level;
        let part_of_other = level_other/total_level;
        let final_color = collux_f32_a_u8((part_of_self * collux_self.0 + part_of_other * collux_other.0, part_of_self * collux_self.1 + part_of_other * collux_other.1, part_of_self * collux_self.2 + part_of_other * collux_other.2));
        let final_level = ((part_of_self * level_self + part_of_other * level_other) * 255.0) as u8;
        Self { level: final_level, r: final_color.0, g: final_color.1, b: final_color.2 }
        */
        Self { level:(a01.to_array()[0] * 255.0).min(255.0) as u8, r, g, b }
    }
}

pub trait Voxel:Clone + Send + Sync + FromBytes + ToBytes + Debug + PartialEq {
    type VT:VoxelType;
    fn voxel_id(&self) -> usize;
    fn orientation(&self) -> u8;
    fn light_level(&self) -> VoxelLight;
    fn thickness(&self) -> u8;
    fn thickness_in_direction(&self, dir:u8) -> u8 {
        let orient = self.orientation();
        let important_part = orient & 0b111;
        let thickness = self.thickness();
        if dir == important_part {
            thickness
        }
        else {
            0
        }
    }
    fn same_outside_as(&self, rhs:&Self) -> bool {
        self.voxel_id() == rhs.voxel_id() && self.orientation() == rhs.orientation() && self.thickness() == rhs.thickness()
    }
    /*fn orientation_converted(&self) -> Orientation {
        // first 3 bits = which side of the voxel is the underside against
        // 000 => base
        // 001 => backside
        // 010 => top side
        // 011 => front side
        // 100 => left side
        // 101 => right side
        // next 2 bits = which horizontal rotation around the underside
        // 00 => no rotation
        // 01 => pi/2
        // 10 => pi
        // 11 => -pi

        let side = match self.orientation() & 7 {
            0 => Orientation::zero(),
            1 => Orientation::new(0.0, PI/2.0, 0.0),
            2 => Orientation::new(0.0, PI, 0.0),
            3 => Orientation::new(0.0, -PI/2.0, 0.0),
            4 => Orientation::new(PI/2.0, PI/2.0, 0.0),
            5 => Orientation::new(-PI/2.0, PI/2.0, 0.0),
            _ => panic!("At the disco")
        };
        side
    }*/
}




pub enum VoxelModel {
    WrappedTexture(usize),
    SpecifiedTexture([usize ; 6]),
    Custom
}

pub trait VoxelType:Clone + Send + Sync + FromBytes + ToBytes {
    fn sides_empty(&self) -> u8;
    fn vertices_taken(&self) -> u8;
    fn kind_of_model(&self) -> VoxelModel;
    fn light_passthrough(&self) -> VoxelLight;
    fn empty_in_direction<V:Voxel>(&self, voxel:&V, neighbor:&V, from:u8, neighbor_type:&Self) -> bool {
        if voxel.orientation() & 0b111 == neighbor.orientation() & 0b111 && (voxel.thickness() <= neighbor.thickness() || neighbor.thickness() == 0) && self.sides_empty() == neighbor_type.sides_empty() && voxel.thickness() > 0 {
            //dbg!(voxel.clone(), neighbor.clone());
            self.empties_with_orientation(DIR_MASK[OPPOSITE_DIR[(voxel.orientation() & 0b111) as usize] as usize], voxel.orientation()) >> (OPPOSITE_DIR[from as usize]) & 1 == 1
        }
        else {
            self.empties_with_orientation(if voxel.thickness() > 0 {0b111110} else {0}, voxel.orientation()) >> (OPPOSITE_DIR[from as usize]) & 1 == 1 || neighbor_type.empties_with_orientation(if neighbor.thickness() > 0 {0b111110} else {0}, neighbor.orientation()) >> (from as u32) & 1 == 1
        }
    }
    fn empty_coming_from<V:Voxel>(&self, voxel:&V, from:u8, orientation:u8) -> bool {
        let rotated_2_empty = self.empties_with_orientation(if voxel.thickness() > 0 {0b111110} else {0}, orientation);
        rotated_2_empty >> (from as u32) & 1 == 1
    }
    fn empties_with_orientation(&self, thickness_mask:u8, orientation:u8) -> u8 {

        let rotat_1 = (orientation >> 3) & 0b00000011;
        let mut self_empty = self.sides_empty();
        self_empty |= thickness_mask;
        let mut rotated_1_empty:u8 = self_empty & 0b00000101;
        for (i, rotates) in ROTATES_ON_1.into_iter().enumerate() {
            let target = LUT_ROTAT_1[rotat_1 as usize][i];
            if target >= rotates {

                rotated_1_empty |= self_empty & (1 << rotates) << (target - rotates)
            }
            else {
                rotated_1_empty |= self_empty & (1 << rotates) >> (rotates - target)
            }
        }
        let mut rotated_2_empty:u8 = 0;
        let rotat_2 = orientation & 0b00000111;
        for i in [0, 1, 2, 3, 4, 5] {
            let target = LUT_ROTAT_2[rotat_2 as usize][i as usize];
            if target >= i {

                rotated_2_empty |= rotated_1_empty & (1 << i) << (target - i)
            }
            else {
                rotated_2_empty |= rotated_1_empty & (1 << i) >> (i - target)
            }
        }
        //if self_empty != rotated_2_empty {
        //    dbg!(self_empty, rotated_2_empty, rotated_1_empty);
        //}   
        
        rotated_2_empty
    }
    fn full_coming_from<V:Voxel>(&self, voxel:&V, from:u8, orientation:u8) -> bool {
        !self.empty_coming_from(voxel, from, orientation)
    }
    fn is_completely_empty(&self) -> bool {
        (self.sides_empty() & 0b00111111) == 0b00111111
    }
    fn sides_empty_bools(&self) -> [bool ; 6] {
        let empty = self.sides_empty();
        let mut bools = [false ; 6];
        for i in 0..6 {
            bools[i] = ((empty >> i) & 1) == 1
        }
        bools
    }
    fn easy_texture(&self) -> usize;
}

const EXPLORATION:[Vec3D<i32> ; 6] = [
    Vec3D::new(0, 0, 1),
    Vec3D::new(1, 0, 0),
    Vec3D::new(0, 0, -1),
    Vec3D::new(-1, 0, 0),
    Vec3D::new(0, -1, 0),
    Vec3D::new(0, 1, 0)
];

const DIR_MASK:[u8 ; 6] = [
    0b00000001,
    0b00000010,
    0b00000100,
    0b00001000,
    0b00010000,
    0b00100000
];

const OPPOSITE_DIR:[u32 ; 6] = [
    2, 3, 0, 1, 5, 4
];

const PERPENDICULAR:[(Vec3D<i32>, Vec3D<i32>) ; 6] = [
    (Vec3D::new(1, 0, 0), Vec3D::new(0, 1, 0)),
    (Vec3D::new(0, 1, 0), Vec3D::new(0, 0, 1)),
    (Vec3D::new(1, 0, 0), Vec3D::new(0, 1, 0)),
    (Vec3D::new(0, 1, 0), Vec3D::new(0, 0, 1)),
    (Vec3D::new(1, 0, 0), Vec3D::new(0, 0, 1)),
    (Vec3D::new(1, 0, 0), Vec3D::new(0, 0, 1)),
];

// first 3 bits = which side of the voxel is the underside(base) against
// 000 => base
// 001 => backside
// 010 => top side
// 011 => front side
// 100 => left side
// 101 => right side
// next 2 bits = which horizontal rotation around the underside
// 00 => no rotation
// 01 => pi/2
// 10 => pi
// 11 => -pi/2

/// sides of the voxel that rotate when rotating around the base, just the sides
const ROTATES_ON_1:[u32 ; 4] = [
    1, 3, 4, 5
];

/// where each horizontal side of the voxel goes once it is rotated around the base 
const LUT_ROTAT_1:[[u32 ; 4] ; 4] = [
    [1, 3, 4, 5],
    [5, 4, 1, 3],
    [3, 1, 5, 4],
    [4, 5, 3, 1]
];

const LUT_ROTAT_2:[[u32 ; 6]; 6] = [
    [0, 1, 2, 3, 4, 5],
    [1, 2, 3, 0, 4, 5],
    [2, 3, 0, 1, 4, 5],
    [3, 0, 1, 2, 4, 5],
    [4, 1, 5, 3, 2, 0],
    [5, 1, 4, 3, 0, 2],
];

const EMPTY_VOXEL:u8 = 0b00111111;


#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct MapChunk<V:Voxel> {
    voxels:Vec<V>,
    only_1_type:Option<V>,
    origin_worldpos:WorldVoxelPos,
    chunk_coord:WorldChunkPos,
    mesh_id:Option<MeshID>,
    mesh_updated:bool,
    mesh_instance:Option<usize>,
    signature:u64
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum GameMapEvent<V:Voxel, G:Generator<V>> {
    UpdateVoxelAt(WorldVoxelPos, V),
    UpdateSetGrid(SetGridUpdate),
    UpdateGenerator(G),
    GeneratorUpdate(G::GeneratorUpdate)
}

impl<ID:Identify, V:Voxel, G:Generator<V>> WorldEvent<GameMap<V, G>, ID> for GameMapEvent<V, G> {
    fn get_source(&self) -> Option<ID> {
        None
    }
    fn should_sync(&self) -> MustSync {
        MustSync::Both
    }
    fn apply_event(self, world:&mut GameMap<V, G>) {
        match self {
            GameMapEvent::UpdateVoxelAt(pos, new_voxel) => {world.get_voxel_at_mut(pos).and_then(|vox| { *vox = new_voxel; None::<()>}); world.modified_this_pos_signal_remesh(pos);},
            GameMapEvent::UpdateSetGrid(set_grid_update) => world.set_grid.apply_update::<VEC_LENGTH, SET_CAPACITY>(set_grid_update),
            GameMapEvent::UpdateGenerator(generator) => world.generator = generator,
            GameMapEvent::GeneratorUpdate(gu) => G::apply_update(gu, world),
        }
    }
}

impl<V:Voxel, G:Generator<V>, ID:Identify> World<ID> for GameMap<V, G> {
    type RB = TestRB;
    type WE = GameMapEvent<V, G>;
    fn update_rendering(&mut self, data:&mut <Self::RB as RenderingBackend>::PreTickData) {
        
    }
}

impl<V:Voxel, G:Generator<V>> Renderable<MantleHandler> for GameMap<V, G> {
    fn do_render_changes(&mut self, render_data:&mut MantleHandler) {
        let mesh_vec = self.mesh_vec;
        if !self.rendering_up_to_date {
            let mut iterator = if self.remesh_fasttrack.len() > 0 {
                self.remesh_fasttrack.clone()
            }
            else {
                self.chunks_map.keys().map(|vector| {vector.clone()}).collect::<Vec<Vec3D<i32>>>()
            };
            self.remesh_fasttrack.clear();
            //let mut test_chunk_tris = 0;
            //let mut test_chunk_id = 0;
            for pos in iterator {
                let around = self.get_chunks_around(pos);
                let chunk = self.get_chunk_at(pos).unwrap();
                if chunk.mesh_updated == false || self.forced_rerender {
                    let mut lods = Vec::new();
                    if self.dims.chunk_height_i == self.dims.chunk_length_i && self.dims.chunk_length_i == self.dims.chunk_width_i && self.dims.chunk_height_i >= 8  {
                        let (first_lod, effective_dirs, mut face_data) = self.get_lod_with_1_step(chunk, around);
                        lods.push(first_lod);
                        const STEPS:[i32 ; 3] = [2, 4, 8];
                        for step in STEPS {
                            lods.push(self.get_lod_with_step(chunk, around, ((step/2) as usize, step), effective_dirs, &mut face_data));
                        }
                    }
                    else {
                        lods.push(self.get_lod_without_step(chunk, around));
                    }
                    lods.reverse();
                    
                    let size = self.dims.chunk_length_f*SQRT_2;
                    let height = self.dims.chunk_height_f;
                    let mut chunk = self.get_chunk_at_mut(pos).unwrap();
                    match chunk.mesh_id.clone() {
                        Some(id) => {
                            render_data.set_or_add_mesh_no_rep(format!("Chunk {} {} {} {}", pos.x, pos.y, pos.z, mesh_vec), lods, Vec::new());
                        },
                        None => {
                            /*let mut local_cool = false;
                            if lod.triangles.len() > test_chunk_tris {
                                test_chunk_tris = lod.triangles.len();
                                local_cool = true;
                            }*/
                            let name = format!("Chunk {} {} {} {}", pos.x, pos.y, pos.z, mesh_vec);
                            let id = render_data.set_or_add_mesh_no_rep(name.clone(), lods, vec![CPUInstanceData {position:get_float_pos(pos) * height, speed:Vec3Df::zero(), rotation:Rotation::from_orientation(Orientation::zero())}]);
                            /*if local_cool {
                                test_chunk_id = id;
                            }*/
                            
                            chunk.mesh_id = Some(MeshID::Name(name)); 
                            chunk.mesh_instance = Some(0)  
                        }
                    }
                    chunk.mesh_updated = true;
                }
            }
            // render_data.meshes.add_instance(MeshInstance::new(Vec3Df::new(5.0, 5.0, 5.0), Orientation::zero(), MeshID::Referenced(test_chunk_id), true, false, true), 3);
            self.rendering_up_to_date = true;
            self.forced_rerender = false;
            // YAHOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOo
        }
    }
}

static TRIS_INDICES_UVS:LazyLock<([[Vec3Df ; 4] ; 6], [usize ; 6], [(f32,f32) ; 4])> = std::sync::LazyLock::new(make_cube_tris);


fn make_cube_tris() -> ([[Vec3Df ; 4] ; 6], [usize ; 6], [(f32,f32) ; 4]) {
    let mut first_face = [
            Vec3D::new(0.5, -0.5, - 0.5),
            Vec3D::new(- 0.5, -0.5, 0.5),
            Vec3D::new(- 0.5, -0.5, - 0.5),
            Vec3D::new(0.5, -0.5, 0.5),
    ];
    let mut indices = [
        2,1,0, 0, 1, 3 
    ];
    let mut texture_uvs = [
        (0.0, 1.0),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 0.0),
    ];
    (
        [

            Rotation::from_orientation(Orientation::new(0.0, 0.0, -PI/2.0)).rotate_array(&first_face), // facing down
            Rotation::from_orientation(Orientation::new(PI/2.0, 0.0, 0.0)).rotate_array(&first_face), // facing front
            Rotation::from_orientation(Orientation::new(0.0, 0.0, PI/2.0)).rotate_array(&first_face), // facing up
            Rotation::from_orientation(Orientation::new(-PI/2.0, 0.0, 0.0)).rotate_array(&first_face), // facing back
            Rotation::from_orientation(Orientation::new(0.0, 0.0, 0.0)).rotate_array(&first_face), // facing right
            Rotation::from_orientation(Orientation::new(PI, 0.0, 0.0)).rotate_array(&first_face), // facing left
            
        ],
        indices,
        texture_uvs,
    )

}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct ChunkDims {
    chunk_width:usize,
    chunk_height:usize,
    chunk_slice_area:usize,
    chunk_length:usize,

    pub chunk_width_i:i32,
    pub chunk_height_i:i32,
    pub chunk_slice_area_i:i32,
    pub chunk_length_i:i32,

    chunk_width_f:f32,
    chunk_height_f:f32,
    chunk_slice_area_f:f32,
    chunk_length_f:f32,
}

impl ChunkDims {
    pub fn get_add_to_other_local(&self, index:usize) -> Vec3D<i32> {
        match index {
            0 => Vec3D::new(0,0, -self.chunk_height_i),
            1 => Vec3D::new(-self.chunk_length_i,0, 0),
            2 => Vec3D::new(0,0, self.chunk_height_i),
            3 => Vec3D::new(self.chunk_length_i,0, 0),
            4 => Vec3D::new(0,self.chunk_width_i, 0),
            5 => Vec3D::new(0,-self.chunk_width_i, 0),
            _ => panic!("Index outside of normal parameters {}", index)
        }
    }
    pub fn new(width:usize, length:usize, height:usize) -> Self {
        Self {
            chunk_width: width,
            chunk_height: height,
            chunk_slice_area: width * length,
            chunk_length: length,

            chunk_width_i: width as i32,
            chunk_height_i: height as i32,
            chunk_slice_area_i: (width * length) as i32,
            chunk_length_i: length as i32,
            
            chunk_width_f: width as f32, 
            chunk_height_f: height as f32,
            chunk_slice_area_f: (width * length) as f32,
            chunk_length_f: length as f32
        }
    }
}



impl<V:Voxel> MapChunk<V> {
    pub fn new(orig_worldpos:WorldVoxelPos, chunk_pos:WorldChunkPos, data:Vec<V>) -> Self {
        let mut only_one = Some(data[0].clone());
        let mut signature = 0_u64;
        let mut moved: i32 = 0;
        let mut last_id = 0;
        for voxel in &data {
            signature ^= signature + ((voxel.voxel_id() as u64) << moved) + (last_id * 120895476097); 
            if let Some(v) = only_one.clone() && v.voxel_id() != voxel.voxel_id() {
                only_one = None;
            }
            moved += 1;
            if moved > 63 {
                moved = 0;
            }
            last_id = voxel.voxel_id() as u64;
        }
        if only_one.is_some() {
            Self { voxels:Vec::new(), origin_worldpos:orig_worldpos, chunk_coord:chunk_pos, mesh_id: None, mesh_updated: false, mesh_instance: None, only_1_type:only_one, signature }
        }
        else {
            Self { voxels:data, origin_worldpos:orig_worldpos, chunk_coord:chunk_pos, mesh_id: None, mesh_updated: false, mesh_instance: None, only_1_type:only_one, signature }
        }
        
    }
    pub fn get_at_local(&self, pos:WorldVoxelPos, dims:&ChunkDims) -> Option<&V> {
        if pos.in_origin_prism(dims.chunk_length_i, dims.chunk_width_i, dims.chunk_height_i) {
            Some(self.get_voxel_data(pos, dims))
        }
        else {
            None
        }
    }
    fn get_voxel_data(&self, pos:WorldVoxelPos, dims:&ChunkDims) -> &V {
        self.only_1_type.as_ref().unwrap_or_else(|| {&self.voxels[pos.x as usize + (pos.y as usize * dims.chunk_length) + (pos.z as usize * dims.chunk_slice_area)]}) 
    }
    fn get_voxel_data_mut(&mut self, pos:WorldVoxelPos, dims:&ChunkDims) -> &mut V {
        self.only_1_type.as_mut().unwrap_or_else(|| {&mut self.voxels[pos.x as usize + (pos.y as usize * dims.chunk_length) + (pos.z as usize * dims.chunk_slice_area)]})
    }
    fn get_at_worldpos(&self, pos:WorldVoxelPos, dims:&ChunkDims) -> Option<&V> {
        self.get_at_local(pos - self.origin_worldpos, dims)
    }
    fn get_voxel_data_at_certain(&self, at:usize) -> &V{
        self.only_1_type.as_ref().unwrap_or_else(|| {&self.voxels[at]})
    }
    fn get_at_worldpos_certain(&self, pos:WorldVoxelPos, dims:&ChunkDims) -> &V {
        self.get_voxel_data(pos - self.origin_worldpos, dims)
    }
    fn get_voxels_around<'a>(&'a self, local_pos:WorldVoxelPos, surrounding_chunks:[Option<&'a MapChunk<V>> ; 6], dims:&ChunkDims) -> [Option<&'a V> ; 6] { 
        let mut voxels = [None ; 6];
        for (i, dir) in EXPLORATION.into_iter().enumerate() {
            match self.get_at_local(local_pos + dir, dims) {
                Some(voxel) => voxels[i] = Some(voxel),
                None => match surrounding_chunks[i] {
                    Some(chunk) => voxels[i] = chunk.get_at_local(local_pos + dir + dims.get_add_to_other_local(i), dims),
                    None => ()
                }
            }
        }
        voxels
    }
    fn get_voxel_in_dir<'a>(&'a self, local_pos:WorldVoxelPos, surrounding_chunks:[Option<&'a MapChunk<V>> ; 6], dims:&ChunkDims, direction:usize) -> Option<&'a V> { 
        let dir = EXPLORATION[direction];
        match self.get_at_local(local_pos + dir, dims) {
            Some(voxel) => Some(voxel),
            None => match surrounding_chunks[direction] {
                Some(chunk) => chunk.get_at_local(local_pos + dir + dims.get_add_to_other_local(direction), dims),
                None => None
            }
        }
    }
    fn mark_for_remesh(&mut self) {
        self.mesh_updated = false;
    }
    fn get_at_worldpos_mut(&mut self, pos:WorldVoxelPos, dims:&ChunkDims) -> Option<&mut V> {
        self.get_at_local_mut(pos - self.origin_worldpos, dims)
    }
    fn get_at_local_mut(&mut self, pos:WorldVoxelPos, dims:&ChunkDims) -> Option<&mut V> {
        if pos.in_origin_prism(dims.chunk_length_i, dims.chunk_width_i, dims.chunk_height_i) {
            Some(self.get_voxel_data_mut(pos, dims))
        }
        else {
            None
        }
    }
    fn get_empty_directions_local<'a>(&self, pos:WorldVoxelPos, dims:&ChunkDims, chunks_around:[Option<&'a MapChunk<V>> ; 6], voxel_types:&Vec<V::VT>) -> u8 {
        let center = self.get_at_local(pos, dims);
        let voxels_around = self.get_voxels_around(pos, chunks_around, dims);
        match center {
            Some(voxel) => {
                let mut self_empty = voxel_types[voxel.voxel_id()].empties_with_orientation(if voxel.thickness() > 0 {0b111110} else {0}, voxel.orientation());
                for (i, dir) in EXPLORATION.iter().enumerate() {
                    match voxels_around[i] {
                        Some(vox_a) => self_empty |= ((voxel_types[vox_a.voxel_id()].empty_coming_from(vox_a, i as u8, vox_a.orientation()) as u8) << i),
                        None => ()
                    } 
                }
                self_empty
                
            },
            None => EMPTY_VOXEL
        }
    }
    fn is_empty_in_direction_local<'a>(&self, pos:WorldVoxelPos, dims:&ChunkDims, chunks_around:[Option<&'a MapChunk<V>> ; 6], voxel_types:&Vec<V::VT>, direction:usize) -> bool {
        let center = self.get_at_local(pos, dims);
        match center {
            Some(voxel) => {
                if voxel.thickness() > 0 {
                    match self.get_voxel_in_dir(pos, chunks_around, dims, direction) {
                        Some(vox_a) => voxel_types[voxel.voxel_id()].empty_in_direction(voxel, vox_a, direction as u8, &voxel_types[vox_a.voxel_id()]),
                        None => false
                    }
                }
                else {
                    let mut self_empty = voxel_types[voxel.voxel_id()].empty_coming_from(voxel, OPPOSITE_DIR[direction as usize] as u8, voxel.orientation());
                    if !self_empty {
                        match self.get_voxel_in_dir(pos, chunks_around, dims, direction) {
                            Some(vox_a) => self_empty || (voxel_types[vox_a.voxel_id()].empty_coming_from(vox_a, direction as u8, vox_a.orientation())),
                            None => self_empty
                        }
                    }
                    else {
                        self_empty
                    }

                }
                
            },
            None => true,
        }
    } 
    pub fn get_lights_local<'a>(&self, pos:WorldVoxelPos, dims:&ChunkDims, chunks_around:[Option<&'a MapChunk<V>> ; 6]) -> [VoxelLight ; 6] {
        let center = self.get_at_local(pos, dims);
        let voxels_around = self.get_voxels_around(pos, chunks_around, dims);
        let mut lights = [VoxelLight::new(0, 0, 0, 0) ; 6];
        match center {
            Some(voxel) => {
                for (i, dir) in EXPLORATION.iter().enumerate() {
                    match voxels_around[i] {
                        Some(vox_a) => lights[i] = vox_a.light_level(),
                        None => ()
                    } 
                }
            },
            None => ()
        }
        lights
    }

    pub fn get_light_in_dir_local<'a>(&self, pos:WorldVoxelPos, dims:&ChunkDims, chunks_around:[Option<&'a MapChunk<V>> ; 6], direction:usize) -> VoxelLight {
        match self.get_voxel_in_dir(pos, chunks_around, dims, direction) {
            Some(voxel) => voxel.light_level(),
            None => VoxelLight::new(0, 0, 0, 0)
        }
    }
    
}

pub type GridUsed = VecGrid;


#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct GameMap<V:Voxel, G:Generator<V>> {
    chunks:Vec<MapChunk<V>>,
    chunks_map:HashMap<WorldChunkPos, usize>,
    dims:ChunkDims,
    voxel_types:Vec<V::VT>,
    mesh_vec:usize,
    rendering_up_to_date:bool,
    forced_rerender:bool,
    min_light_levels:(u8,u8,u8),
    remesh_fasttrack:Vec<WorldChunkPos>,
    pub set_grid:GridUsed,
    pub generator:G
}

pub type WorldChunkPos = Vec3D<i32>;
pub type WorldVoxelPos = Vec3D<i32>;

pub fn get_chunk_pos_i(dims:&ChunkDims, pos:Vec3D<i32>) -> WorldChunkPos {
    Vec3D::new(
        (pos.x.div_floor(dims.chunk_length_i)),
        (pos.y.div_floor(dims.chunk_width_i)),
        (pos.z.div_floor(dims.chunk_height_i)),
    )
}

fn multiply_corresponding_nonzero(mut scaler:Vec3Df, dir:Vec3Df) -> Vec3Df {
    if dir.x != 0.0 {
        scaler.x *= dir.x;
        scaler
    }
    else if dir.y != 0.0 {
        scaler.y *= dir.y;
        scaler
    }
    else {
        scaler.z *= dir.z;
        scaler
    }
}

pub fn get_minimum_nudge<V:Voxel, G:Generator<V>>(end:Vec3Df, nudge:Vec3Df, world:&GameMap<V, G>) -> Vec3Df {
    let xy_zeroed_out = end + Vec3Df::new(0.0, 0.0, nudge.z);
    if world.is_voxel_solid(get_voxel_pos(xy_zeroed_out)) {
        let stepped_z_nudge = end + Vec3Df::new(0.0, 0.0, nudge.z + 1.0);
        if world.is_voxel_solid(get_voxel_pos(stepped_z_nudge)) {
            let x_zeroed = end + Vec3Df::new(0.0, nudge.y, nudge.z);
            let y_zeroed = end + Vec3Df::new(nudge.x, 0.0, nudge.z);
            let z_zeroed = end + Vec3Df::new(nudge.x, nudge.y, 0.0);
            if !world.is_voxel_solid(get_voxel_pos(x_zeroed)) {
                Vec3Df::new(0.0, nudge.y, nudge.z)
            }
            else if !world.is_voxel_solid(get_voxel_pos(y_zeroed)) {
                Vec3Df::new(nudge.x, 0.0, nudge.z)
            }
            else if !world.is_voxel_solid(get_voxel_pos(z_zeroed)) {
                Vec3Df::new(nudge.x, nudge.y, 0.0)
            }
            else {
                nudge
            }
        }
        else {
            Vec3Df::new(0.0, 0.0, nudge.z + 0.5)
        }
        
        
    }
    else {
        Vec3Df::new(0.0, 0.0, nudge.z)
    }
}

impl<V:Voxel, G:Generator<V>> GameMap<V, G> {
    pub fn new(expected_chunks:usize, dims:ChunkDims, voxel_types:Vec<V::VT>, min_light_levels:(u8,u8,u8), mesh_vec:usize, generator:G, grid_slot_scale:f32, grid_size:i32) -> Self {
        Self { generator, chunks:Vec::with_capacity(expected_chunks), chunks_map: HashMap::with_capacity(expected_chunks), dims, voxel_types, forced_rerender:false, min_light_levels, mesh_vec, rendering_up_to_date: false, remesh_fasttrack:Vec::with_capacity(16), set_grid:GridUsed::new(grid_slot_scale, Vec3D::all_ones() * grid_size, Vec3D::all_ones() * grid_size, SETGRID_SIZE) }
    }

    
    pub fn full_collision(&self, pos:Vec3Df, speed_nudge:Vec3Df) -> Option<Collision<V>> {
        self.generator.full_collision(pos, speed_nudge).or_else(|| {
            match self.get_voxel_at(get_voxel_pos(pos)) {
                Some(voxel) => Some(Collision { surface_normal: Vec3Df::new(0.0, 0.0, 1.0), minimum_nudge: get_minimum_nudge(pos, speed_nudge, self), voxel: voxel.clone(), position:pos }),
                None => None
            }
        })
    }
    pub fn simple_collision_hinted(&self, pos:Vec3Df, hint:&mut Option<VoxelGetHint>) -> bool {
        self.generator.simple_collision(pos) || self.is_voxel_solid_hinted(get_voxel_pos(pos), hint)
    }
    pub fn simple_collision(&self, pos:Vec3Df) -> bool {
        self.generator.simple_collision(pos) || self.is_voxel_solid(get_voxel_pos(pos))
    }
    fn get_lod_without_step(&self, chunk:&MapChunk<V>, around:[Option<&MapChunk<V>> ; 6]) -> ApiLod {
        let mut vertices = Vec::with_capacity(600);
        let mut indices = Vec::with_capacity(600);
        let mut lod = ApiLod::new(get_world_textures_list(), vertices, indices);
        for x in 0..self.dims.chunk_length_i {
            for y in 0..self.dims.chunk_width_i {
                for z in 0..self.dims.chunk_height_i {

                    let voxel = chunk.get_at_local(Vec3D::new(x, y, z), &self.dims).unwrap();
                    if !&self.voxel_types[voxel.voxel_id()].is_completely_empty() {
                        let empty_dirs = chunk.get_empty_directions_local(Vec3D::new(x, y, z), &self.dims, around, &self.voxel_types);
                        let lights = chunk.get_lights_local(Vec3D::new(x, y, z), &self.dims, around);
                        //dbg!(Vec3D::new(x, y, z));
                        for (i, mask) in DIR_MASK.iter().enumerate() {
                            /*if empty_dirs != 0 {
                                dbg!(empty_dirs);
                            }*/
                            if empty_dirs & mask  == *mask { //if a direction is empty (hint:it's a me, Mario)
                                let mut points = TRIS_INDICES_UVS.0[i];
                                let indices = TRIS_INDICES_UVS.1;
                                let uvs = TRIS_INDICES_UVS.2;
                                let start_index = lod.vertex_data.len();
                                let mut level = (lights[i].level.max(50) as f32) / 255.0;
                                let mut converted_collux = collux_u8_a_f32((lights[i].r, lights[i].g, lights[i].b));
                                let mut finished_collux = collux_f32_a_u8((converted_collux.0 * level, converted_collux.1 * level, converted_collux.2 * level));
                                finished_collux = (finished_collux.0.max(self.min_light_levels.0), finished_collux.1.max(self.min_light_levels.1), finished_collux.2.max(self.min_light_levels.2));
                                points = points + Vec3D::new(x as f32, y as f32, z as f32); 
                                let points:Vec<CPUVertexData> = points.iter().enumerate().map(|p| {
                                    CPUVertexData {
                                        position:p.1.clone(),
                                        texture_id:self.voxel_types[voxel.voxel_id()].easy_texture() as u8,
                                        u:TRIS_INDICES_UVS.2[p.0].0,
                                        v:TRIS_INDICES_UVS.2[p.0].1
                                    }
                                }).collect();
                                lod.add_points([points[0].clone(),points[1].clone(),points[2].clone(),points[3].clone()]);
                                lod.add_triangle(
                                    TrianglePoint::new(
                                        indices[0] + start_index,
                                        uvs[indices[0]].0,
                                        uvs[indices[0]].1,
                                        finished_collux.0,
                                        finished_collux.1,
                                        finished_collux.2
                                    ),
                                    TrianglePoint::new(
                                        indices[1] + start_index,
                                        uvs[indices[1]].0,
                                        uvs[indices[1]].1,
                                        finished_collux.0,
                                        finished_collux.1,
                                        finished_collux.2
                                    ),
                                    TrianglePoint::new(
                                        indices[2] + start_index,
                                        uvs[indices[2]].0,
                                        uvs[indices[2]].1,
                                        finished_collux.0,
                                        finished_collux.1,
                                        finished_collux.2
                                    )
                                );
                                lod.add_triangle(
                                    TrianglePoint::new(
                                        indices[3] + start_index,
                                        uvs[indices[3]].0,
                                        uvs[indices[3]].1,
                                        finished_collux.0,
                                        finished_collux.1,
                                        finished_collux.2
                                    ),
                                    TrianglePoint::new(
                                        indices[4] + start_index,
                                        uvs[indices[4]].0,
                                        uvs[indices[4]].1,
                                        finished_collux.0,
                                        finished_collux.1,
                                        finished_collux.2
                                    ),
                                    TrianglePoint::new(
                                        indices[5] + start_index,
                                        uvs[indices[5]].0,
                                        uvs[indices[5]].1,
                                        finished_collux.0,
                                        finished_collux.1,
                                        finished_collux.2
                                    )
                                );
                            }
                        }
                    }
                }
            }
        }
        lod
    }

    fn get_face_data_for_dir_first_pass(
        &self,
        (x,y,z):(i32,i32,i32),
        (i,mask):(usize,u8),
        chunk:&MapChunk<V>,
        around:[Option<&MapChunk<V>> ; 6],
        taken_dirs:&mut Vec<u8>,
        effective_dirs:&mut [bool ; 6]
    ) -> Option<(u32, (u8,u8,u8), u8)> {
        match chunk.get_at_local(Vec3D::new(x, y, z), &self.dims){
            Some(voxel) => if !&self.voxel_types[voxel.voxel_id()].is_completely_empty() {
                if chunk.is_empty_in_direction_local(Vec3D::new(x, y, z), &self.dims, around, &self.voxel_types, i) {
                    let full_texture = self.voxel_types[voxel.voxel_id()].easy_texture() as u32;
                    let light = chunk.get_light_in_dir_local(Vec3D::new(x, y, z), &self.dims, around, i);
                    let mut level = (light.level.max(50) as f32) * 0.00392156862;
                    let mut converted_collux = collux_u8_a_f32((light.r, light.g, light.b));
                    let mut finished_collux = collux_f32_a_u8((converted_collux.0 * level, converted_collux.1 * level, converted_collux.2 * level));
                    finished_collux = (finished_collux.0.max(self.min_light_levels.0), finished_collux.1.max(self.min_light_levels.1), finished_collux.2.max(self.min_light_levels.2));
                    effective_dirs[i] = true;
                    let thickness = chunk.get_at_local(Vec3D::new(x, y, z), &self.dims).unwrap().thickness_in_direction(i as u8);
                    Some((full_texture, finished_collux, thickness))
                }
                else {
                    None
                }
            }
            else {
                None
            },
            None => return None
        }
    }


    fn get_face_data_for_dir(
        &self,
        (x,y,z):(i32,i32,i32),
        (i,mask):(usize,u8),
        (prev_step,step):(usize, i32),
        face_data:&mut Vec<[Option<(u32, (u8,u8,u8), u8)> ; 6]>
    ) -> Option<(u32, (u8,u8,u8), u8)> {
            for dx in (x..x+step).step_by(prev_step) {
                for dy in (y..y+step).step_by(prev_step) {
                    for dz in (z..z+step).step_by(prev_step) {
                        if Vec3D::new(dx, dy, dz).in_origin_prism(self.dims.chunk_length_i, self.dims.chunk_width_i, self.dims.chunk_height_i) {
                            if let Some(data) = face_data[dx as usize + (dy as usize * self.dims.chunk_length) + (dz as usize * self.dims.chunk_slice_area)][i] {
                                return Some(data)
                            }
                        }
                        else {
                            return None
                        }
                        
                        
                    }
                }
            }
            None
        
    }
    fn aggregation_start_step_1(&self, chunk:&MapChunk<V>, around:[Option<&MapChunk<V>> ; 6], start:(i32,i32,i32), dir:usize, mask:u8, taken_dirs:&mut Vec<u8>, effective_dirs:&mut [bool ; 6], face_data:&mut Vec<[Option<(u32, (u8,u8,u8), u8)> ; 6]>) -> Option<(u32, (u8,u8,u8), u8)> {
        let position = Vec3D::new(start.0, start.1, start.2);
        let value = self.get_face_data_for_dir_first_pass((position.x, position.y, position.z), (dir, mask), chunk, around, taken_dirs, effective_dirs);
        let index = (position.x as usize + (position.y as usize * self.dims.chunk_length) + (position.z as usize * self.dims.chunk_slice_area));
        taken_dirs[index] |= mask;
        face_data[index][dir] = value;
        value
    }

    fn aggregation_start_step_n(&self, (prev_step, step):(usize,i32), start:(i32,i32,i32), dir:usize, mask:u8, taken_dirs:&mut Vec<u8>, face_data:&mut Vec<[Option<(u32, (u8,u8,u8), u8)> ; 6]>) -> Option<(u32, (u8,u8,u8), u8)> {
        let position = Vec3D::new(start.0, start.1, start.2);
        let value = self.get_face_data_for_dir((position.x, position.y, position.z), (dir, mask), (prev_step, step), face_data);
        let index = (position.x as usize + (position.y as usize * self.dims.chunk_length) + (position.z as usize * self.dims.chunk_slice_area));
        taken_dirs[index] |= mask;
        face_data[index][dir] = value;
        value
    }

    fn is_area_all_same_in_dir_step_1(&self, chunk:&MapChunk<V>, around:[Option<&MapChunk<V>> ; 6], start:(i32,i32,i32), dir:usize, mask:u8, (area, area_start):((i32, i32), (i32,i32)), taken_dirs:&mut Vec<u8>, mut same:Option<(u32, (u8,u8,u8), u8)>, vox:V, effective_dirs:&mut [bool ; 6], face_data:&mut Vec<[Option<(u32, (u8,u8,u8), u8)> ; 6]>) -> Option<(u32, (u8,u8,u8), u8)> {
        let (mut first_dir, mut second_dir) = PERPENDICULAR[dir].clone();
        for first in area_start.0..area.0 {
            for second in area_start.1..area.1 {
                let position = Vec3D::new(start.0, start.1, start.2) + first_dir * first + second_dir * second;
                let value = self.get_face_data_for_dir_first_pass((position.x, position.y, position.z), (dir, mask), chunk, around, taken_dirs, effective_dirs);
                let local_vox = chunk.get_at_local(position, &self.dims).cloned();
                if same != value || (local_vox.is_some() && !vox.same_outside_as(&local_vox.unwrap())) {
                    return None;
                }
            }
        }
        for first in area_start.0..area.0 {
            for second in area_start.1..area.1 {
                let position = Vec3D::new(start.0, start.1, start.2) + first_dir * first + second_dir * second;
                let index = (position.x as usize + (position.y as usize * self.dims.chunk_length) + (position.z as usize * self.dims.chunk_slice_area));
                taken_dirs[index] |= mask;
                face_data[index][dir] = same;

            }
        }
        same
    }

    fn is_area_all_same_in_dir(&self, chunk:&MapChunk<V>, around:[Option<&MapChunk<V>> ; 6], (prev_step, step):(usize,i32), start:(i32,i32,i32), dir:usize, mask:u8, (area, area_start):((i32, i32), (i32,i32)), scaler:&Vec3Df, taken_dirs:&mut Vec<u8>, mut same:Option<(u32, (u8,u8,u8), u8)>, face_data:&mut Vec<[Option<(u32, (u8,u8,u8), u8)> ; 6]>) -> Option<(u32, (u8,u8,u8), u8)> {
        
        let (mut first_dir, mut second_dir) = PERPENDICULAR[dir].clone();
        first_dir *= step;
        second_dir *= step;
        for first in area_start.0..area.0 {
            for second in area_start.1..area.1 {
                let position = Vec3D::new(start.0, start.1, start.2) + first_dir * first + second_dir * second;
                let value = self.get_face_data_for_dir((position.x, position.y, position.z), (dir, mask), (prev_step, step), face_data);
                if same.is_none() {
                    same = value
                }
                else if same != value {
                    return None;
                }
            }
        }
        for first in area_start.0..area.0 {
            for second in area_start.1..area.1 {
                let position = Vec3D::new(start.0, start.1, start.2) + first_dir * first + second_dir * second;
                let index = (position.x as usize + (position.y as usize * self.dims.chunk_length) + (position.z as usize * self.dims.chunk_slice_area));
                taken_dirs[index] |= mask;
                face_data[index][dir] = same;
            }
        }
        same
    }
    
    fn get_lod_with_1_step(&self, chunk:&MapChunk<V>, around:[Option<&MapChunk<V>> ; 6]) -> (ApiLod, [bool ; 6], Vec<[Option<(u32, (u8,u8,u8), u8)> ; 6]>) {
        if let Some(voxel) = chunk.only_1_type.clone() && self.voxel_types[voxel.voxel_id()].is_completely_empty() {
            (ApiLod::new(Vec::new(), Vec::new(), Vec::new()), [false ; 6], vec![])
        }
        else {
            const THICKNESS_DIR:[Vec3Df ; 6] = [
                Vec3D::new(0.0, 0.0, 1.0),
                Vec3D::new(1.0, 0.0, 0.0),
                Vec3D::new(0.0, 0.0, -1.0),
                Vec3D::new(-1.0, 0.0, 0.0),
                Vec3D::new(0.0, -1.0, 0.0),
                Vec3D::new(0.0, 1.0, 0.0)
            ];
            const CUBE_VERTICES:[Vec3Df ; 8] = [
                Vec3D::new(0.5, -0.5, - 0.5),
                Vec3D::new(- 0.5, -0.5, 0.5),
                Vec3D::new(- 0.5, -0.5, - 0.5),
                Vec3D::new(0.5, -0.5, 0.5),
                Vec3D::new(0.5, 0.5, - 0.5),
                Vec3D::new(- 0.5, 0.5, 0.5),
                Vec3D::new(- 0.5, 0.5, - 0.5),
                Vec3D::new(0.5, 0.5, 0.5)
            ];
            /* 
            
const EXPLORATION:[Vec3D<i32> ; 6] = [
    Vec3D::new(0, 0, 1),
    Vec3D::new(1, 0, 0),
    Vec3D::new(0, 0, -1),
    Vec3D::new(-1, 0, 0),
    Vec3D::new(0, -1, 0),
    Vec3D::new(0, 1, 0)
];
            */
            const USED_VERTICES:[[usize ; 4] ; 6] = [
                [1, 3, 5, 7],
                [0, 3, 4, 7],
                [0, 2, 4, 6],
                [1, 2, 5, 6],
                [0, 1, 2, 3],
                [4, 5, 6, 7],
            ];
            const INDICES:[[usize ; 6] ; 6] = [
                [0,1,3, 0, 2, 3],
                [0,1,3, 0, 2, 3],
                [0,1,3, 0, 2, 3],
                [0,1,3, 0, 2, 3],
                [2,1,0, 0, 1, 3],
                [2,1,0, 0, 1, 3], 
            ];
            const TEXTURE_UVS:[[(f32,f32) ; 4] ; 6] = [
                [
                    (0.0, 0.0),
                    (1.0, 0.0),
                    (0.0, 1.0),
                    (1.0, 1.0),
                ],
                [
                    (0.0, 0.0),
                    (1.0, 0.0),
                    (0.0, 1.0),
                    (1.0, 1.0),
                ],
                [
                    (0.0, 0.0),
                    (1.0, 0.0),
                    (0.0, 1.0),
                    (1.0, 1.0),
                ],
                [
                    (0.0, 0.0),
                    (1.0, 0.0),
                    (0.0, 1.0),
                    (1.0, 1.0),
                ],
                [
                    (0.0, 1.0),
                    (1.0, 0.0),
                    (1.0, 1.0),
                    (0.0, 0.0),
                ],
                [
                    (0.0, 1.0),
                    (1.0, 0.0),
                    (1.0, 1.0),
                    (0.0, 0.0),
                ],
            ];
            let mut vertices = Vec::with_capacity(256);
            let mut indices = Vec::with_capacity(256);
            let mut lod = ApiLod::new_simple(get_world_textures_list(), vertices, indices);
            let mut taken_dirs = vec![0 ; self.dims.chunk_height * self.dims.chunk_length * self.dims.chunk_width];
            let mut kept_dirs = Vec::with_capacity(6);
            let mut effective_dirs = [false ; 6];
            let mut face_data_vec = vec![[None ; 6] ; self.dims.chunk_height * self.dims.chunk_length * self.dims.chunk_width];
            let scaler = Vec3Df::all_ones();
            if let Some(voxel) = chunk.only_1_type.clone() && !self.voxel_types[voxel.voxel_id()].is_completely_empty() {
                for (i, ch) in around.iter().enumerate() {
                    if let Some(chunk2) = ch {
                        if let Some(voxel) = chunk2.only_1_type.clone() && self.voxel_types[voxel.voxel_id()].is_completely_empty() {
                            kept_dirs.push((i, DIR_MASK[i]));
                            effective_dirs[i] = true;
                        }
                        else if let None = chunk2.only_1_type.clone() {
                            kept_dirs.push((i, DIR_MASK[i]));
                            effective_dirs[i] = true;
                        }
                    }
                }
            }
            else {
                for (i, ch) in around.iter().enumerate() {
                    kept_dirs.push((i, DIR_MASK[i]));
                }
            }
            
            // println!("Started {} {} {}", chunk.chunk_coord.x, chunk.chunk_coord.y, chunk.chunk_coord.z);
            for x in 0..self.dims.chunk_length_i {
                for y in 0..self.dims.chunk_width_i {
                    for z in 0..self.dims.chunk_height_i {
                        let mut dirs_data = Vec::with_capacity(6);
                        let mut all_points = CUBE_VERTICES.clone();
                        for (i, mask) in kept_dirs.iter() {
                            if taken_dirs[(x as usize + (y as usize * self.dims.chunk_length) + (z as usize * self.dims.chunk_slice_area))] & *mask == *mask {
                                let vox = chunk.get_at_local(Vec3D::new(x, y, z), &self.dims).cloned().unwrap();
                                let thickness = vox.thickness_in_direction(*i as u8);
                                if thickness != 0 {
                                    for index in USED_VERTICES[*i] {
                                        all_points[index] -= THICKNESS_DIR[*i] * ((255 - thickness) as f32)/255.0;
                                    }
                                }
                                continue
                            }
                            else {
                                let mut first_dir_greed = 1;
                                let mut second_dir_greed = 1;
                                let face_data = self.aggregation_start_step_1(chunk, around, (x,y,z), *i, *mask, &mut taken_dirs, &mut effective_dirs, &mut face_data_vec);
                                let vox = chunk.get_at_local(Vec3D::new(x, y, z), &self.dims).cloned().unwrap();

                                while (first_dir_greed + 1) < self.dims.chunk_height_i && (second_dir_greed + 1) < self.dims.chunk_height_i && face_data.is_some() {
                                    
                                    //println!("{} {} {}", first_dir_greed, second_dir_greed, step);
                                    let mut first_test_face = self.is_area_all_same_in_dir_step_1(chunk, around, (x,y,z), *i, *mask, ((first_dir_greed + 1, second_dir_greed), (first_dir_greed, 0)), &mut taken_dirs, face_data, vox.clone(), &mut effective_dirs, &mut face_data_vec);
                                    if first_test_face.is_some() {
                                        first_dir_greed += 1;
                                    }

                                    let mut second_test_face = self.is_area_all_same_in_dir_step_1(chunk, around, (x,y,z), *i, *mask, ((first_dir_greed, second_dir_greed + 1), (0, second_dir_greed)), &mut taken_dirs, face_data, vox.clone(), &mut effective_dirs, &mut face_data_vec);
                                    if second_test_face.is_some() {
                                        second_dir_greed += 1;
                                    }
                                    if first_test_face.is_none() && second_test_face.is_none() {
                                        break;
                                    }
                                }
                                match face_data {
                                    Some((full_texture, finished_collux, thickness)) => {
                                        dirs_data.push((*i, full_texture, finished_collux, thickness, first_dir_greed, second_dir_greed));
                                        if thickness != 0 {
                                            for index in USED_VERTICES[*i] {
                                                all_points[index] -= THICKNESS_DIR[*i] * ((255 - thickness) as f32)/255.0;
                                            }
                                        }

                                        
                                    },
                                    None => ()
                                }
                            }
                        }
                        for (i, full_texture, finished_collux, thickness, first_dir_greed, second_dir_greed) in dirs_data {
                            let u_s = first_dir_greed as f32;
                            let v_s = second_dir_greed as f32;
                            let (mut f_dir, mut s_dir) = PERPENDICULAR[i].clone();
                            let scaler = multiply_corresponding_nonzero(multiply_corresponding_nonzero(scaler, Vec3D::new(f_dir.x as f32, f_dir.y as f32, f_dir.z as f32) * u_s), Vec3D::new(s_dir.x as f32, s_dir.y as f32, s_dir.z as f32) * v_s);

                            let start_index = lod.vertex_data.len();
                            let indices = INDICES[i];
                            let uvs = TEXTURE_UVS[i];
                            let mut points = [Vec3D::zero() ; 4];
                            let mut j = 0;
                            for index in USED_VERTICES[i] {
                                points[j] = all_points[index];
                                j += 1;
                            }
                            points = points * scaler + (Vec3D::new(x as f32, y as f32, z as f32)) + ((Vec3Df::all_ones() * 0.5).component_product(&scaler) - Vec3Df::all_ones() * 0.5);
                            let points:Vec<CPUVertexData> = points.iter().enumerate().map(|p| {
                                CPUVertexData {
                                    position:p.1.clone(),
                                    texture_id:full_texture as u8,
                                    u:uvs[p.0].0,
                                    v:uvs[p.0].1
                                }
                            }).collect();
                            lod.add_points([points[0].clone(),points[1].clone(),points[2].clone(),points[3].clone()]);
                            lod.add_triangle(
                                TrianglePoint::new(
                                    indices[0] + start_index,
                                    uvs[indices[0]].0 * u_s,
                                    uvs[indices[0]].1 * v_s,
                                    finished_collux.0,
                                    finished_collux.1,
                                    finished_collux.2
                                ),
                                TrianglePoint::new(
                                    indices[1] + start_index,
                                    uvs[indices[1]].0 * u_s,
                                    uvs[indices[1]].1 * v_s,
                                    finished_collux.0,
                                    finished_collux.1,
                                    finished_collux.2
                                ),
                                TrianglePoint::new(
                                    indices[2] + start_index,
                                    uvs[indices[2]].0 * u_s,
                                    uvs[indices[2]].1 * v_s,
                                    finished_collux.0,
                                    finished_collux.1,
                                    finished_collux.2
                                )
                            );
                            lod.add_triangle(
                                TrianglePoint::new(
                                    indices[3] + start_index,
                                    uvs[indices[3]].0 * u_s,
                                    uvs[indices[3]].1 * v_s,
                                    finished_collux.0,
                                    finished_collux.1,
                                    finished_collux.2
                                ),
                                TrianglePoint::new(
                                    indices[4] + start_index,
                                    uvs[indices[4]].0 * u_s,
                                    uvs[indices[4]].1 * v_s,
                                    finished_collux.0,
                                    finished_collux.1,
                                    finished_collux.2
                                ),
                                TrianglePoint::new(
                                    indices[5] + start_index,
                                    uvs[indices[5]].0 * u_s,
                                    uvs[indices[5]].1 * v_s,
                                    finished_collux.0,
                                    finished_collux.1,
                                    finished_collux.2
                                )
                            );
                        }
                    }
                }
            }
            (lod, effective_dirs, face_data_vec)
        }
        
    }
    fn get_lod_with_step(&self, chunk:&MapChunk<V>, around:[Option<&MapChunk<V>> ; 6], (prev_step, step):(usize, i32), effective_dirs:[bool ; 6], vec_face_data:&mut Vec<[Option<(u32, (u8,u8,u8), u8)> ; 6]>) -> ApiLod {
        if let Some(voxel) = chunk.only_1_type.clone() && self.voxel_types[voxel.voxel_id()].is_completely_empty() {
            ApiLod::new(Vec::new(), Vec::new(), Vec::new())
        }
        else {
            let capacity = (256/step) as usize;
            let mut vertices = Vec::with_capacity(capacity);
            let mut indices = Vec::with_capacity(capacity);
            let mut lod = ApiLod::new(get_world_textures_list(), vertices, indices );
            let scaler = Vec3D::new(step as f32, step as f32, step as f32);
            let float_step = step as f32;
            let mut taken_dirs = vec![0 ; self.dims.chunk_height * self.dims.chunk_length * self.dims.chunk_width];
            let mut kept_dirs = Vec::with_capacity(6);
            if let Some(voxel) = chunk.only_1_type.clone() && !self.voxel_types[voxel.voxel_id()].is_completely_empty() {
                for (i, ch) in around.iter().enumerate() {
                    if let Some(chunk2) = ch {
                        if let Some(voxel) = chunk2.only_1_type.clone() && self.voxel_types[voxel.voxel_id()].is_completely_empty() {
                            kept_dirs.push((i, DIR_MASK[i]));
                        }
                        else if let None = chunk2.only_1_type.clone() {
                            kept_dirs.push((i, DIR_MASK[i]));
                        }
                    }
                }
            }
            else {
                for (i, ch) in around.iter().enumerate() {
                    if effective_dirs[i] {
                        kept_dirs.push((i, DIR_MASK[i]));
                    }
                }
            }
            
            // println!("Started {} {} {}", chunk.chunk_coord.x, chunk.chunk_coord.y, chunk.chunk_coord.z);
            for x in (0..self.dims.chunk_length_i).step_by(step as usize) {
                for y in (0..self.dims.chunk_width_i).step_by(step as usize) {
                    for z in (0..self.dims.chunk_height_i).step_by(step as usize) {
                        for (i, mask) in kept_dirs.iter() {
                            if taken_dirs[(x as usize + (y as usize * self.dims.chunk_length) + (z as usize * self.dims.chunk_slice_area))] & *mask == *mask {
                                continue
                            }
                            else {
                                let mut first_dir_greed = 1;
                                let mut second_dir_greed = 1;
                                let face_data = self.aggregation_start_step_n((prev_step, step), (x,y,z), *i, *mask, &mut taken_dirs, vec_face_data);

                                while (first_dir_greed + 1) * step < self.dims.chunk_height_i && (second_dir_greed + 1) * step < self.dims.chunk_height_i && face_data.is_some() {
                                    
                                    //println!("{} {} {}", first_dir_greed, second_dir_greed, step);
                                    let mut first_test_face = self.is_area_all_same_in_dir(chunk, around, (prev_step, step), (x,y,z), *i, *mask, ((first_dir_greed + 1, second_dir_greed), (first_dir_greed, 0)), &scaler, &mut taken_dirs, face_data, vec_face_data);
                                    if first_test_face.is_some() {
                                        first_dir_greed += 1;
                                    }

                                    let mut second_test_face = self.is_area_all_same_in_dir(chunk, around, (prev_step, step), (x,y,z), *i, *mask, ((first_dir_greed, second_dir_greed + 1), (0, second_dir_greed)), &scaler, &mut taken_dirs, face_data, vec_face_data);
                                    if second_test_face.is_some() {
                                        second_dir_greed += 1;
                                    }
                                    if first_test_face.is_none() && second_test_face.is_none() {
                                        break;
                                    }
                                }
                                match face_data {
                                    Some((full_texture, finished_collux, thickness)) => {
                                        let u_s = first_dir_greed as f32;
                                        let v_s = second_dir_greed as f32;
                                        let (mut f_dir, mut s_dir) = PERPENDICULAR[*i].clone();
                                        let scaler = multiply_corresponding_nonzero(multiply_corresponding_nonzero(scaler, Vec3D::new(f_dir.x as f32, f_dir.y as f32, f_dir.z as f32) * u_s), Vec3D::new(s_dir.x as f32, s_dir.y as f32, s_dir.z as f32) * v_s);

                                        let start_index = lod.vertex_data.len();
                                        let indices = TRIS_INDICES_UVS.1;
                                        let uvs = TRIS_INDICES_UVS.2;
                                        let mut points = TRIS_INDICES_UVS.0[*i];
                                        points = points * scaler + (Vec3D::new(x as f32, y as f32, z as f32)) + ((Vec3Df::all_ones() * 0.5).component_product(&scaler) - Vec3Df::all_ones() * 0.5);
                                        let points:Vec<CPUVertexData> = points.iter().enumerate().map(|p| {
                                            CPUVertexData {
                                                position:p.1.clone(),
                                                texture_id:full_texture as u8,
                                                u:TRIS_INDICES_UVS.2[p.0].0,
                                                v:TRIS_INDICES_UVS.2[p.0].1
                                            }
                                        }).collect();
                                        lod.add_points([points[0].clone(),points[1].clone(),points[2].clone(),points[3].clone()]);
                                        lod.add_triangle(
                                            TrianglePoint::new(
                                                indices[0] + start_index,
                                                uvs[indices[0]].0 * u_s,
                                                uvs[indices[0]].1 * v_s,
                                                finished_collux.0,
                                                finished_collux.1,
                                                finished_collux.2
                                            ),
                                            TrianglePoint::new(
                                                indices[1] + start_index,
                                                uvs[indices[1]].0 * u_s,
                                                uvs[indices[1]].1 * v_s,
                                                finished_collux.0,
                                                finished_collux.1,
                                                finished_collux.2
                                            ),
                                            TrianglePoint::new(
                                                indices[2] + start_index,
                                                uvs[indices[2]].0 * u_s,
                                                uvs[indices[2]].1 * v_s,
                                                finished_collux.0,
                                                finished_collux.1,
                                                finished_collux.2
                                            )
                                        );
                                        lod.add_triangle(
                                            TrianglePoint::new(
                                                indices[3] + start_index,
                                                uvs[indices[3]].0 * u_s,
                                                uvs[indices[3]].1 * v_s,
                                                finished_collux.0,
                                                finished_collux.1,
                                                finished_collux.2
                                            ),
                                            TrianglePoint::new(
                                                indices[4] + start_index,
                                                uvs[indices[4]].0 * u_s,
                                                uvs[indices[4]].1 * v_s,
                                                finished_collux.0,
                                                finished_collux.1,
                                                finished_collux.2
                                            ),
                                            TrianglePoint::new(
                                                indices[5] + start_index,
                                                uvs[indices[5]].0 * u_s,
                                                uvs[indices[5]].1 * v_s,
                                                finished_collux.0,
                                                finished_collux.1,
                                                finished_collux.2
                                            )
                                        );
                                    },
                                    None => ()
                                }
                            }
                        }
                    }
                }
            }
            lod
        }
        
    }
    pub fn does_chunk_exist(&self, chunk:WorldChunkPos) -> bool {
        self.chunks_map.contains_key(&chunk)
    }
    pub fn set_min_light_levels(&mut self, mins:(u8,u8,u8)) {
        self.min_light_levels = mins;
    }
    pub fn force_rerender(&mut self) {
        self.rendering_up_to_date = false;
        self.forced_rerender = true;
    }
    pub fn modified_this_pos_signal_remesh(&mut self, pos:WorldVoxelPos) {
        let mut must_re_render = false;
        let dims = self.dims.clone();
        let mut add_to_fasttrack = Vec::with_capacity(7);
        self.get_chunk_at_mut(self.get_chunk_pos_i(pos)).and_then(|chunk| {chunk.mark_for_remesh(); add_to_fasttrack.push(get_chunk_pos_i(&dims, pos)); must_re_render = true; Some(1_i32)});
        for dir in EXPLORATION {
            self.get_chunk_at_mut(self.get_chunk_pos_i(pos + dir)).and_then(|chunk| {chunk.mark_for_remesh(); add_to_fasttrack.push(get_chunk_pos_i(&dims, pos + dir)); must_re_render = true; Some(1_i32)});
        }
        for add in add_to_fasttrack {
            self.remesh_fasttrack.push(add);
        }
        self.rendering_up_to_date = !must_re_render;
    }
    pub fn generate_chunk<F:FnMut(Vec3D<i32>) -> V>(&mut self, chunk_pos:WorldChunkPos, func:&mut F) {
        let mut chunk_data = Vec::with_capacity(self.dims.chunk_slice_area * self.dims.chunk_height);
        let mut orig_worldpos = Vec3D::new(chunk_pos.x * self.dims.chunk_length_i, chunk_pos.y * self.dims.chunk_width_i, chunk_pos.z * self.dims.chunk_height_i);
        for z in orig_worldpos.z..orig_worldpos.z + self.dims.chunk_height_i {

            //println!("Doing z={}", z);
            let in_z  = (z - orig_worldpos.z) as usize * self.dims.chunk_slice_area;
            for y in orig_worldpos.y..orig_worldpos.y + self.dims.chunk_width_i {
                let in_y  = (y - orig_worldpos.y) as usize * self.dims.chunk_length;
                for x in orig_worldpos.x..orig_worldpos.x + self.dims.chunk_length_i {
                    chunk_data.push(func(Vec3D::new(x, y, z)));
                }
            }
        }
        let chunk_id = self.chunks.len();
        self.chunks.push(MapChunk::new(orig_worldpos, chunk_pos, chunk_data));
        self.chunks_map.insert(chunk_pos, chunk_id);
    }
    pub fn get_all_chunk_pos(&self) -> Vec<Vec3D<i32>> {
        self.chunks_map.clone().keys().map(|vector| {vector.clone()}).collect::<Vec<Vec3D<i32>>>()
    }
    pub fn generate_chunks<F:FnMut(Vec3D<i32>) -> V>(&mut self, start:WorldChunkPos, end:WorldChunkPos, func:&mut F) {
        self.set_grid = GridUsed::new(self.set_grid.get_slot_scale(), start * self.dims.chunk_length_i, end * self.dims.chunk_length_i, SET_CAPACITY);
        for xc in start.x..end.x {
            for yc in start.y..end.y {
                for zc in start.z..end.z {
                    self.generate_chunk(Vec3D::new(xc, yc, zc), func);
                }
            }
        }
    }
    pub fn change_mesh_vec(&mut self, new_vec:usize) {
        self.mesh_vec = new_vec;
    }

    pub fn get_chunk_at(&self, chunk_pos:WorldChunkPos) -> Option<&MapChunk<V>> {
        self.chunks_map.get(&chunk_pos).and_then(|idx| {Some(&self.chunks[*idx])})
    }
    pub fn get_chunk_at_mut(&mut self, chunk_pos:WorldChunkPos) -> Option<&mut MapChunk<V>> {
        self.chunks_map.get(&chunk_pos).and_then(|idx| {Some(&mut self.chunks[*idx])})
    }
    pub fn get_chunks_around(&self, chunk_pos:WorldChunkPos) -> [Option<&MapChunk<V>> ; 6] {
        let mut chunks = [None ; 6];
        for (i, offset) in EXPLORATION.into_iter().enumerate() {
            chunks[i] = self.get_chunk_at(chunk_pos + offset)
        }
        chunks
    }
    pub fn get_chunk_and_surrounding(&self, chunk_pos:WorldChunkPos) -> (Option<&MapChunk<V>>, [Option<&MapChunk<V>> ; 6]) {
        (
            self.get_chunk_at(chunk_pos),
            self.get_chunks_around(chunk_pos)
        )
    }
    pub fn get_chunk_pos(&self, pos:Vec3Df) -> WorldChunkPos {
        Vec3D::new(
            (pos.x/self.dims.chunk_length_f).trunc() as i32,
            (pos.y/self.dims.chunk_width_f).trunc() as i32,
            (pos.z/self.dims.chunk_height_f - 1.0).trunc() as i32,
        )
    }
    pub fn get_chunk_pos_i(&self, pos:Vec3D<i32>) -> WorldChunkPos {
        Vec3D::new(
            (pos.x.div_floor(self.dims.chunk_length_i)),
            (pos.y.div_floor(self.dims.chunk_width_i)),
            (pos.z.div_floor(self.dims.chunk_height_i)),
        )
    }
    pub fn get_chunk_dims_vector(&self) -> Vec3D<i32> {
        Vec3D::new(
            self.dims.chunk_length_i,
            self.dims.chunk_width_i,
            self.dims.chunk_height_i,
        )
    }
    pub fn is_voxel_solid(&self, voxel:WorldVoxelPos) -> bool {
        self.get_voxel_at(voxel).is_some_and(|voxel| {!self.get_voxel_types()[voxel.voxel_id()].is_completely_empty()})
    }
    pub fn is_voxel_solid_hinted(&self, voxel:WorldVoxelPos, hint:&mut Option<VoxelGetHint>) -> bool {
        match self.get_voxel_at_hinted(voxel, hint) {
            Some(v) => !self.get_voxel_types()[v.voxel_id()].is_completely_empty(),
            None => false
        }
    }
    pub fn get_voxel_at_mut(&mut self, voxel:WorldVoxelPos) -> Option<&mut V> {
        let dims = self.dims.clone();
        self.get_chunk_at_mut(self.get_chunk_pos_i(voxel)).and_then(|chunk| {chunk.get_at_worldpos_mut(voxel, &dims)})
    }
    pub fn get_voxel_at(&self, voxel:WorldVoxelPos) -> Option<&V> {
        let dims = self.dims.clone();
        //self.in_world_positions.get(&voxel).and_then(|(chunk, exact)| {Some(self.chunks.get(chunk).unwrap().get_voxel_data_at_certain(*exact))})
        self.get_chunk_at(self.get_chunk_pos_i(voxel)).and_then(|chunk| {chunk.get_at_worldpos(voxel, &dims)})
    }
    pub fn get_type_of_voxel_at(&self, voxel:WorldVoxelPos) -> Option<&V::VT> {
        let dims = self.dims.clone();
        self.get_chunk_at(self.get_chunk_pos_i(voxel)).and_then(|chunk| {chunk.get_at_worldpos(voxel, &dims)}).and_then(|voxel| {Some(&self.voxel_types[voxel.voxel_id()])})
    }
    pub fn get_voxel_at_hinted(&self, voxel:WorldVoxelPos, hint:&mut Option<VoxelGetHint>) -> Option<&V> {
        match hint {
            Some(given_hint) => match self.chunks[given_hint.get_chunk()].get_at_worldpos(voxel, &self.dims) {
                Some(vox) => Some(vox),
                None => {
                    *hint = None;
                    self.get_voxel_at_hinted(voxel, hint)
                }
            },
            None => self.chunks_map.get(&self.get_chunk_pos_i(voxel)).and_then(|idx| {
                *hint = Some(VoxelGetHint::new(*idx));
                Some(self.chunks[*idx].get_at_worldpos_certain(voxel, &self.dims))
            })
        }
    }
    pub fn get_ceiling_at(&self, pos:WorldVoxelPos, margin:i32) -> WorldVoxelPos {
        for z in ((pos.z - margin)..(pos.z + margin)).rev() {
            match self.get_voxel_at(Vec3D::new(pos.x, pos.y, z)) {
                Some(voxel) => if !self.get_voxel_types()[voxel.voxel_id()].is_completely_empty() {
                    return Vec3D::new(pos.x, pos.y, z)
                },
                None => ()
            }
        }
        pos
    }
    pub fn get_voxel_types(&self) -> &Vec<V::VT> {
        &self.voxel_types
    }
}

pub fn get_voxel_pos(pos:Vec3Df) -> WorldVoxelPos {
    Vec3D::new(
        pos.x as i32,
        pos.y as i32,
        pos.z as i32
    )
}

pub fn get_float_pos(pos:WorldVoxelPos) -> Vec3Df {
    Vec3D::new(
        pos.x as f32,
        pos.y as f32,
        pos.z as f32
    )
}

pub fn get_f64_pos(pos:WorldVoxelPos) -> Vec3D<f64> {
    Vec3D::new(
        pos.x as f64,
        pos.y as f64,
        pos.z as f64
    )
}



pub trait Generator<V:Voxel>: Clone + ToBytes + FromBytes + PartialEq {
    const PROVIDES_COLLISION:bool;
    type GeneratorUpdate:Clone + ToBytes + FromBytes + PartialEq; 
    fn generate(&self, pos:WorldVoxelPos) -> V;
    fn simple_collision(&self, pos:Vec3Df) -> bool {
        self.full_collision(pos, Vec3Df::zero()).is_some()
    }
    fn full_collision(&self, pos:Vec3Df, speed_nudge:Vec3Df) -> Option<Collision<V>>;
    fn apply_update(update:Self::GeneratorUpdate, world:&mut GameMap<V, Self>);
    fn must_sync(update:&Self::GeneratorUpdate) -> MustSync;
    fn get_chunks_to_show(&self, camera:&Camera) -> Vec<WorldChunkPos>;
    fn get_chunks_to_hide(&self, camera:&Camera) -> Vec<WorldChunkPos>;
    fn get_chunks_to_unmesh(&self, camera:&Camera) -> Vec<WorldChunkPos>;
}


pub struct Collision<V:Voxel> {
    pub surface_normal:Vec3Df,
    pub minimum_nudge:Vec3Df,
    pub position:Vec3Df,
    pub voxel:V
}

#[derive(Clone, Copy, ToBytes, FromBytes, PartialEq, Debug)]
pub struct VoxelGetHint {
    chunk:u32,
}

impl VoxelGetHint {
    fn new(chunk:usize) -> Self {
        Self { chunk: chunk as u32 }
    }
    pub fn get_chunk(&self) -> usize {
        self.chunk as usize
    }
}