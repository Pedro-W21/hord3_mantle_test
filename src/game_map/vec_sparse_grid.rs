use std::{sync::atomic::{AtomicBool, Ordering}, collections::{HashSet}};

use hord3::horde::geometry::vec3d::{Vec3D, Vec3Df};
use to_from_bytes::{FromBytes, ToBytes};
use to_from_bytes_derive::{ToBytes, FromBytes};

use crate::{game_entity::colliders::AABB, game_map::{SET_CAPACITY, turbo_set::{TurboSet, TurboSetIterator}}};

use super::{get_float_pos, WorldVoxelPos};



#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct SparseGrid<T: FromBytes + ToBytes + PartialEq + Clone> {
    grid:Vec<Option<usize>>,
    data:Vec<T>,
    slot_scale:f32,
    inv_scale:f32,
    prism_start:Vec3D<f32>,
    prism_end:Vec3D<f32>,
    length:usize,
    width:usize,
    height:usize,
    f_l:f32,
    f_w:f32,
    f_h:f32,
    base_area:usize,
    per_slot:usize,
}

impl<T: FromBytes + ToBytes + PartialEq + Clone> SparseGrid<T> {
    pub fn new(slot_scale:f32, start:WorldVoxelPos, end:WorldVoxelPos, per_slot:usize) -> Self {
        let world_length = (end.x - start.x) as usize;
        let world_width = (end.y - start.y) as usize;
        let world_height = (end.z - start.z) as usize;
        let length = (world_length / (slot_scale as usize)) + 1;
        let width = (world_width / (slot_scale as usize)) + 1;
        let height = (world_height / (slot_scale as usize)) + 1;
        /*let mut has_data_grid = Vec::with_capacity(length * width * height);
        for i in 0..has_data_grid.capacity() {
            has_data_grid.push(AtomicBool::new(false));
        }*/
        let inv_scale = 1.0/slot_scale;
        let prism_start = get_float_pos(start).mul_floor(inv_scale);
        let prism_end = get_float_pos(end).mul_floor(inv_scale);
        Self {prism_end, slot_scale, prism_start, grid: vec![None ; length * width * height], base_area:width * length, data: Vec::with_capacity(length * width * height), inv_scale: 1.0/slot_scale, length, width, height, f_h:height as f32, f_l:length as f32, f_w:width as f32, per_slot }
    }
    pub fn add_data_to_slot(&mut self, data:T, slot:usize) {
        let index = self.data.len();
        for i in 0..self.per_slot {
            self.data.push(data.clone());
        }
        self.grid[slot] = Some(index);
    }
    /*pub fn signal_data_add(&self, slot:usize) -> bool { // true if you can send an event
        if let Ok(false) = self.has_data_grid[slot].compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
            true
        }
        else {
            false
        }
    }*/
    pub fn is_in(&self, point:&Vec3Df) -> bool {
        let scaled_point = point * self.inv_scale;
        scaled_point.in_origin_prism(self.f_l, self.f_w, self.f_h)
    }
    pub fn get(&self, (x,y,z):(usize,usize,usize)) -> Option<usize> {
        self.grid[x + y * self.length + z * self.base_area]
    }
    pub fn get_if_in(&self, point:&Vec3Df) -> Option<usize> {
        match (point.mul_floor(self.inv_scale) - self.prism_start).to_usize_if_in_orig_prism(self.f_l, self.f_w, self.f_h) {
            Some(triple_coord) => self.get(triple_coord),
            None => None
        }
    }
    pub fn get_data_if_in(&self, point:&Vec3Df) -> Option<&T> {
        match self.get_if_in(point) {
            Some(id) => Some(&self.data[id]),
            None => None
        }
    }
    pub fn get_data_u_coords(&self, point:(usize,usize,usize)) -> Option<&T> {
        match self.get(point) {
            Some(id) => Some(&self.data[id]),
            None => None
        }
    }
    pub fn get_data_mut_if_in(&mut self, point:&Vec3Df) -> Option<&mut T> {
        match self.get_if_in(point) {
            Some(id) => Some(&mut self.data[id]),
            None => None
        }
    }
    pub fn get_data_mut_u_coords(&mut self, point:(usize,usize,usize)) -> Option<&mut T> {
        match self.get(point) {
            Some(id) => Some(&mut self.data[id]),
            None => None
        }
    }
    pub fn vec3D_to_usize(&self, point:&Vec3Df) -> Option<usize> {
        match (point.mul_floor(self.inv_scale) - self.prism_start).to_usize_if_in_orig_prism(self.f_l, self.f_w, self.f_h) {
            Some(u_point) => Some(u_point.0 + u_point.1 * self.length + u_point.2 * self.base_area),
            None => None
        }
    }
    pub fn get_data_id_for_slot(&self, slot:usize) -> Option<usize> {
        self.grid[slot]
    }
    pub fn modify_data_at<F:Fn(&mut T)>(&mut self, data_id:usize, func:&F) {
        func(&mut self.data[data_id])
    }
}

pub type SetGrid = SparseGrid<HashSet<usize>>;
 

impl SetGrid {
    pub fn get_slot_scale(&self) -> f32 {
        self.slot_scale
    }
    pub fn get_iter_from_to<'a>(&'a self, start:Vec3Df, stop:Vec3Df, index_in_vec:usize, size:f32) -> SGIterator<'a> {
        
        let (mut start, mut stop) = AABB::new(start, stop).get_both_points();
        start -= Vec3Df::all_ones() * size * 2.0;
        stop += Vec3Df::all_ones() * size * 2.0 ;

        let mut start_usize = (start.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        let mut stop_usize = (stop.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
 /* 
        start_usize.0 -= 1.clamp(0, start_usize.0);
        start_usize.1 -= 1.clamp(0, start_usize.1);
        start_usize.2 -= 1.clamp(0, start_usize.2);

        stop_usize.0 = (stop_usize.0 + 1).clamp(0, self.length - 1);
        stop_usize.1 = (stop_usize.1 + 1).clamp(0, self.width - 1);
        stop_usize.2 = (stop_usize.2 + 1).clamp(0, self.height - 1);
*/
        let x_limit = stop_usize.0 - start_usize.0;
        let y_limit = stop_usize.1 - start_usize.1;
        let z_limit = stop_usize.2 - start_usize.2;

        let end_of_line_add = self.length - x_limit;
        let end_of_layer_add = start_usize.0 + start_usize.1 * self.length + (start_usize.2 + 1) * self.base_area - (stop_usize.0 + stop_usize.1 * self.length + start_usize.2 * self.base_area);
        let mut iteraor = SGIterator { grid: self, current_set: None, end_of_line_add, end_of_layer_add, current_pos: (0,0,0), x_limit, y_limit, z_limit, current_index: start_usize.0 + start_usize.1 * self.length + start_usize.2 * self.base_area, index_in_vec };
        iteraor.initial_iterator_update();
        iteraor
    }
    fn add_to_hashset<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize, vec_index:usize, add:usize) {
        let index = self.make_sure_data_exists_at::<VEC_LENGTH, SET_CAPACITY>(grid_slot);
        self.data[index + vec_index].insert(add);
    }
    fn remove_from_hashset<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize, vec_index:usize, remove:usize) {
        let index = self.make_sure_data_exists_at::<VEC_LENGTH, SET_CAPACITY>(grid_slot);
        self.data[index + vec_index].remove(&remove);
    }
    fn make_sure_data_exists_at<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize) -> usize {
        match self.grid[grid_slot] {
            Some(data_id) => data_id,
            None => {
                let len = self.data.len();
                for i in 0..VEC_LENGTH {
                    self.data.push(HashSet::with_capacity(SET_CAPACITY));
                }
                self.grid[grid_slot] = Some(len);
                len
            }
        }
    }
    pub fn apply_update<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, update:SetGridUpdate) {
        match update {
            SetGridUpdate::AddToSet { grid_slot, vec_index, add } => self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_slot, vec_index, add),
            SetGridUpdate::RemoveFromSet { grid_slot, vec_index, remove } => self.remove_from_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_slot, vec_index, remove),
            SetGridUpdate::MoveFromTo { grid_start, grid_end, vec_index, id } => {
                self.remove_from_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_start, vec_index, id);
                self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_end, vec_index, id);
            }
        }
    }
    fn exists_at(&self, grid_slot:usize, vec_index:usize, id:usize) -> bool {
        match self.grid[grid_slot] {
            Some(data) => self.data[data + vec_index].contains(&id),
            None => false
        }
    }
    fn len_at(&self, grid_slot:usize, vec_index:usize) {
        match self.grid[grid_slot] {
            Some(data) => {dbg!(self.data[data + vec_index].len());},
            None => ()
        }
    }
    pub fn get_point_move_update(&self, start:&Vec3Df, end:&Vec3Df, id:usize, vec_index:usize) -> Option<SetGridUpdate> {
        match self.vec3D_to_usize(start) {
            Some(grid_start) => match self.vec3D_to_usize(end) {
                Some(grid_end) => if grid_start != grid_end {
                    Some(SetGridUpdate::MoveFromTo { grid_start, grid_end, vec_index, id })
                }
                else if !self.exists_at(grid_end, vec_index, id) {
                    Some(SetGridUpdate::AddToSet { grid_slot: grid_end, vec_index, add:id })
                }
                else {
                    None
                },
                None => Some(SetGridUpdate::RemoveFromSet { grid_slot: grid_start, vec_index, remove:id })
            },
            None => match self.vec3D_to_usize(end) {
                Some(grid_end) => Some(SetGridUpdate::AddToSet { grid_slot: grid_end, vec_index, add:id }),
                None => None
            }
        }
    }
    pub fn add_to_prism_slow<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, start:&Vec3Df, stop:&Vec3Df, id:usize, vec_index:usize) {
        let (start, stop) = AABB::new(*start, *stop).get_both_points();

        let start_usize = (start.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        let stop_usize = (stop.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        for x in start_usize.0..=stop_usize.0 {
            for y in start_usize.1..=stop_usize.1 {
                for z in start_usize.2..=stop_usize.2 {
                    self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(x + y * self.length + z * self.base_area, vec_index, id);
                }
            }
        }
    }
}

#[derive(Clone, ToBytes, FromBytes, Debug, PartialEq)]
pub enum SetGridUpdate {
    AddToSet {grid_slot:usize, vec_index:usize, add:usize},
    RemoveFromSet {grid_slot:usize, vec_index:usize, remove:usize},
    MoveFromTo {grid_start:usize, grid_end:usize, vec_index:usize, id:usize}
}



pub struct SGIterator<'a> {
    grid:&'a SparseGrid<HashSet<usize>>,
    current_set:Option<std::collections::hash_set::Iter<'a, usize>>,
    end_of_line_add:usize,
    end_of_layer_add:usize,
    current_pos:(usize,usize,usize),
    x_limit:usize,
    y_limit:usize,
    z_limit:usize,
    current_index:usize,
    index_in_vec:usize
}

impl<'a> SGIterator<'a> {
    fn update_pos(&mut self) -> bool {
        if self.current_pos.0 == self.x_limit {
            if self.current_pos.1 == self.y_limit {
                if self.current_pos.2 == self.z_limit {
                    /**/
                    false
                }
                else {
                    self.current_pos.0 = 0;
                    self.current_pos.1 = 0;
                    self.current_pos.2 += 1;
                    self.current_index += self.end_of_layer_add;
                    true
                }
            }
            else {
                self.current_pos.0 = 0;
                self.current_pos.1 += 1;
                self.current_index += self.end_of_line_add;
                true
            }
        }
        else {
            self.current_pos.0 += 1;
            self.current_index += 1;
            true
        }
    }
    fn next_slot(&mut self) -> Option<usize> {
        while self.update_pos() {
            if let Some(value) = self.update_iterator() {
                return Some(value)
            }
        }
        if !self.update_pos() {
            match &self.current_set {
                Some(set) => {
                    None
                },
                None => match self.grid.grid[self.current_index] {
                    Some(data_id) => {
                        let set = &self.grid.data[data_id + self.index_in_vec];
                        if set.len() > 0 {
                            let mut new_iterator = set.iter();
                            let value = new_iterator.next().unwrap();
                            self.current_set = Some(new_iterator);
                            Some(*value)
                        }
                        else {
                            None
                        }
                    },
                    None => {
                        None
                    }
                }
            }
        }
        else {
            None
        }
    }
    fn update_iterator(&mut self) -> Option<usize> {
        match self.grid.grid[self.current_index] {
            Some(data_id) => {
                let set = &self.grid.data[data_id + self.index_in_vec];
                if set.len() > 0 {
                    let mut new_iterator = set.iter();
                    let value = new_iterator.next().unwrap();
                    self.current_set = Some(new_iterator);
                    Some(*value)
                }
                else {
                    None
                }
            },
            None => None
        }
    }
    fn initial_iterator_update(&mut self) {
        match self.grid.grid[self.current_index] {
            Some(data_id) => {
                let new_iterator = self.grid.data[data_id + self.index_in_vec].iter();
                self.current_set = Some(new_iterator);
            },
            None => ()
        }
    }
} 

impl<'a> Iterator for SGIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current_set.as_mut() {
            Some(iterator) => match iterator.next() {
                Some(data) => {
                    Some(*data)
                },
                None => self.next_slot(),
            },
            None => self.next_slot(),
        }
    }
}



pub type VecGrid = SparseGrid<Vec<usize>>;
 

impl VecGrid {
    pub fn get_slot_scale(&self) -> f32 {
        self.slot_scale
    }
    pub fn get_iter_from_to<'a>(&'a self, start:Vec3Df, stop:Vec3Df, index_in_vec:usize, size:f32) -> VGIterator<'a> {
        let (mut start, mut stop) = AABB::new(start, stop).get_both_points();
        start -= Vec3Df::all_ones() * size * 2.0;
        stop += Vec3Df::all_ones() * size * 2.0 ;

        let mut start_usize = (start.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        let mut stop_usize = (stop.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
 /* 
        start_usize.0 -= 1.clamp(0, start_usize.0);
        start_usize.1 -= 1.clamp(0, start_usize.1);
        start_usize.2 -= 1.clamp(0, start_usize.2);

        stop_usize.0 = (stop_usize.0 + 1).clamp(0, self.length - 1);
        stop_usize.1 = (stop_usize.1 + 1).clamp(0, self.width - 1);
        stop_usize.2 = (stop_usize.2 + 1).clamp(0, self.height - 1);
*/
        let x_limit = stop_usize.0 - start_usize.0;
        let y_limit = stop_usize.1 - start_usize.1;
        let z_limit = stop_usize.2 - start_usize.2;

        let end_of_line_add = self.length - x_limit;
        let end_of_layer_add = start_usize.0 + start_usize.1 * self.length + (start_usize.2 + 1) * self.base_area - (stop_usize.0 + stop_usize.1 * self.length + start_usize.2 * self.base_area);
        let mut iteraor = VGIterator { grid: self, current_set: None, end_of_line_add, end_of_layer_add, current_pos: (0,0,0), x_limit, y_limit, z_limit, current_index: start_usize.0 + start_usize.1 * self.length + start_usize.2 * self.base_area, index_in_vec };
        iteraor.initial_iterator_update();
        iteraor
    }
    fn add_to_hashset<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize, vec_index:usize, add:usize) {
        let index = self.make_sure_data_exists_at::<VEC_LENGTH, SET_CAPACITY>(grid_slot);
        self.data[index + vec_index].push(add);
    }
    fn remove_from_hashset<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize, vec_index:usize, remove:usize) {
        let index = self.make_sure_data_exists_at::<VEC_LENGTH, SET_CAPACITY>(grid_slot);
        let remove_index = self.data[index + vec_index].iter().enumerate().find(|(i, val)| {**val == remove}).unwrap().0;
        self.data[index + vec_index].remove(remove_index);
    }
    fn make_sure_data_exists_at<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize) -> usize {
        match self.grid[grid_slot] {
            Some(data_id) => data_id,
            None => {
                let len = self.data.len();
                for i in 0..VEC_LENGTH {
                    self.data.push(Vec::with_capacity(SET_CAPACITY));
                }
                self.grid[grid_slot] = Some(len);
                len
            }
        }
    }
    pub fn apply_update<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, update:SetGridUpdate) {
        match update {
            SetGridUpdate::AddToSet { grid_slot, vec_index, add } => self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_slot, vec_index, add),
            SetGridUpdate::RemoveFromSet { grid_slot, vec_index, remove } => self.remove_from_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_slot, vec_index, remove),
            SetGridUpdate::MoveFromTo { grid_start, grid_end, vec_index, id } => {
                self.remove_from_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_start, vec_index, id);
                self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_end, vec_index, id);
            }
        }
    }
    fn exists_at(&self, grid_slot:usize, vec_index:usize, id:usize) -> bool {
        match self.grid[grid_slot] {
            Some(data) => self.data[data + vec_index].contains(&id),
            None => false
        }
    }
    fn len_at(&self, grid_slot:usize, vec_index:usize) {
        match self.grid[grid_slot] {
            Some(data) => {dbg!(self.data[data + vec_index].len());},
            None => ()
        }
    }
    pub fn get_point_move_update(&self, start:&Vec3Df, end:&Vec3Df, id:usize, vec_index:usize) -> Option<SetGridUpdate> {
        match self.vec3D_to_usize(start) {
            Some(grid_start) => match self.vec3D_to_usize(end) {
                Some(grid_end) => if grid_start != grid_end {
                    Some(SetGridUpdate::MoveFromTo { grid_start, grid_end, vec_index, id })
                }
                else if !self.exists_at(grid_end, vec_index, id) {
                    Some(SetGridUpdate::AddToSet { grid_slot: grid_end, vec_index, add:id })
                }
                else {
                    None
                },
                None => Some(SetGridUpdate::RemoveFromSet { grid_slot: grid_start, vec_index, remove:id })
            },
            None => match self.vec3D_to_usize(end) {
                Some(grid_end) => Some(SetGridUpdate::AddToSet { grid_slot: grid_end, vec_index, add:id }),
                None => None
            }
        }
    }
    pub fn add_to_prism_slow<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, start:&Vec3Df, stop:&Vec3Df, id:usize, vec_index:usize) {
        let (start, stop) = AABB::new(*start, *stop).get_both_points();

        let start_usize = (start.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        let stop_usize = (stop.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        for x in start_usize.0..=stop_usize.0 {
            for y in start_usize.1..=stop_usize.1 {
                for z in start_usize.2..=stop_usize.2 {
                    self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(x + y * self.length + z * self.base_area, vec_index, id);
                }
            }
        }
    }
}

#[derive(Clone, ToBytes, FromBytes, Debug, PartialEq)]
pub enum VecGridUpdate {
    AddToSet {grid_slot:usize, vec_index:usize, add:usize},
    RemoveFromSet {grid_slot:usize, vec_index:usize, remove:usize},
    MoveFromTo {grid_start:usize, grid_end:usize, vec_index:usize, id:usize}
}



pub struct VGIterator<'a> {
    grid:&'a SparseGrid<Vec<usize>>,
    current_set:Option<std::slice::Iter<'a, usize>>,
    end_of_line_add:usize,
    end_of_layer_add:usize,
    current_pos:(usize,usize,usize),
    x_limit:usize,
    y_limit:usize,
    z_limit:usize,
    current_index:usize,
    index_in_vec:usize
}

impl<'a> VGIterator<'a> {
    fn update_pos(&mut self) -> bool {
        if self.current_pos.0 == self.x_limit {
            if self.current_pos.1 == self.y_limit {
                if self.current_pos.2 == self.z_limit {
                    /**/
                    false
                }
                else {
                    self.current_pos.0 = 0;
                    self.current_pos.1 = 0;
                    self.current_pos.2 += 1;
                    self.current_index += self.end_of_layer_add;
                    true
                }
            }
            else {
                self.current_pos.0 = 0;
                self.current_pos.1 += 1;
                self.current_index += self.end_of_line_add;
                true
            }
        }
        else {
            self.current_pos.0 += 1;
            self.current_index += 1;
            true
        }
    }
    fn next_slot(&mut self) -> Option<usize> {
        while self.update_pos() {
            if let Some(value) = self.update_iterator() {
                return Some(value)
            }
        }
        if !self.update_pos() {
            match &self.current_set {
                Some(set) => {
                    None
                },
                None => match self.grid.grid[self.current_index] {
                    Some(data_id) => {
                        let set = &self.grid.data[data_id + self.index_in_vec];
                        if set.len() > 0 {
                            let mut new_iterator = set.iter();
                            let value = new_iterator.next().unwrap();
                            self.current_set = Some(new_iterator);
                            Some(*value)
                        }
                        else {
                            None
                        }
                    },
                    None => {
                        None
                    }
                }
            }
        }
        else {
            None
        }
    }
    fn update_iterator(&mut self) -> Option<usize> {
        match self.grid.grid[self.current_index] {
            Some(data_id) => {
                let set = &self.grid.data[data_id + self.index_in_vec];
                if set.len() > 0 {
                    let mut new_iterator = set.iter();
                    let value = new_iterator.next().unwrap();
                    self.current_set = Some(new_iterator);
                    Some(*value)
                }
                else {
                    None
                }
            },
            None => None
        }
    }
    fn initial_iterator_update(&mut self) {
        match self.grid.grid[self.current_index] {
            Some(data_id) => {
                let new_iterator = self.grid.data[data_id + self.index_in_vec].iter();
                self.current_set = Some(new_iterator);
            },
            None => ()
        }
    }
} 

impl<'a> Iterator for VGIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current_set.as_mut() {
            Some(iterator) => match iterator.next() {
                Some(data) => {
                    Some(*data)
                },
                None => self.next_slot(),
            },
            None => self.next_slot(),
        }
    }
}



pub type TurboGrid = SparseGrid<TurboSet>;
 

impl TurboGrid {
    pub fn get_slot_scale(&self) -> f32 {
        self.slot_scale
    }
    pub fn get_iter_from_to<'a>(&'a self, start:Vec3Df, stop:Vec3Df, index_in_vec:usize, size:f32) -> TGIterator<'a> {
        let (mut start, mut stop) = AABB::new(start, stop).get_both_points();
        start -= Vec3Df::all_ones() * size * 2.0;
        stop += Vec3Df::all_ones() * size * 2.0 ;

        let mut start_usize = (start.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        let mut stop_usize = (stop.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
 /* 
        start_usize.0 -= 1.clamp(0, start_usize.0);
        start_usize.1 -= 1.clamp(0, start_usize.1);
        start_usize.2 -= 1.clamp(0, start_usize.2);

        stop_usize.0 = (stop_usize.0 + 1).clamp(0, self.length - 1);
        stop_usize.1 = (stop_usize.1 + 1).clamp(0, self.width - 1);
        stop_usize.2 = (stop_usize.2 + 1).clamp(0, self.height - 1);
*/
        let x_limit = stop_usize.0 - start_usize.0;
        let y_limit = stop_usize.1 - start_usize.1;
        let z_limit = stop_usize.2 - start_usize.2;

        let end_of_line_add = self.length - x_limit;
        let end_of_layer_add = start_usize.0 + start_usize.1 * self.length + (start_usize.2 + 1) * self.base_area - (stop_usize.0 + stop_usize.1 * self.length + start_usize.2 * self.base_area);
        let mut iteraor = TGIterator { grid: self, current_set: None, end_of_line_add, end_of_layer_add, current_pos: (0,0,0), x_limit, y_limit, z_limit, current_index: start_usize.0 + start_usize.1 * self.length + start_usize.2 * self.base_area, index_in_vec };
        iteraor.initial_iterator_update();
        iteraor
    }
    fn add_to_hashset<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize, vec_index:usize, add:usize) {
        let index = self.make_sure_data_exists_at::<VEC_LENGTH, SET_CAPACITY>(grid_slot);
        self.data[index + vec_index].insert(add);
    }
    fn remove_from_hashset<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize, vec_index:usize, remove:usize) {
        let index = self.make_sure_data_exists_at::<VEC_LENGTH, SET_CAPACITY>(grid_slot);
        let remove_index = self.data[index + vec_index].remove(remove);
    }
    fn make_sure_data_exists_at<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:usize) -> usize {
        match self.grid[grid_slot] {
            Some(data_id) => data_id,
            None => {
                let len = self.data.len();
                for i in 0..VEC_LENGTH {
                    self.data.push(TurboSet::new(SET_CAPACITY, 2000));
                }
                self.grid[grid_slot] = Some(len);
                len
            }
        }
    }
    pub fn apply_update<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, update:SetGridUpdate) {
        match update {
            SetGridUpdate::AddToSet { grid_slot, vec_index, add } => self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_slot, vec_index, add),
            SetGridUpdate::RemoveFromSet { grid_slot, vec_index, remove } => self.remove_from_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_slot, vec_index, remove),
            SetGridUpdate::MoveFromTo { grid_start, grid_end, vec_index, id } => {
                self.remove_from_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_start, vec_index, id);
                self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_end, vec_index, id);
            }
        }
    }
    fn exists_at(&self, grid_slot:usize, vec_index:usize, id:usize) -> bool {
        match self.grid[grid_slot] {
            Some(data) => self.data[data + vec_index].contains(id),
            None => false
        }
    }
    fn len_at(&self, grid_slot:usize, vec_index:usize) {
        match self.grid[grid_slot] {
            Some(data) => {dbg!(self.data[data + vec_index].len());},
            None => ()
        }
    }
    pub fn get_point_move_update(&self, start:&Vec3Df, end:&Vec3Df, id:usize, vec_index:usize) -> Option<SetGridUpdate> {
        match self.vec3D_to_usize(start) {
            Some(grid_start) => match self.vec3D_to_usize(end) {
                Some(grid_end) => if grid_start != grid_end {
                    Some(SetGridUpdate::MoveFromTo { grid_start, grid_end, vec_index, id })
                }
                else if !self.exists_at(grid_end, vec_index, id) {
                    Some(SetGridUpdate::AddToSet { grid_slot: grid_end, vec_index, add:id })
                }
                else {
                    None
                },
                None => Some(SetGridUpdate::RemoveFromSet { grid_slot: grid_start, vec_index, remove:id })
            },
            None => match self.vec3D_to_usize(end) {
                Some(grid_end) => Some(SetGridUpdate::AddToSet { grid_slot: grid_end, vec_index, add:id }),
                None => None
            }
        }
    }
    pub fn add_to_prism_slow<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, start:&Vec3Df, stop:&Vec3Df, id:usize, vec_index:usize) {
        let (start, stop) = AABB::new(*start, *stop).get_both_points();

        let start_usize = (start.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        let stop_usize = (stop.mul_floor(self.inv_scale) - self.prism_start).to_u_orig_prism_clamped(self.f_l, self.f_w, self.f_h);
        for x in start_usize.0..=stop_usize.0 {
            for y in start_usize.1..=stop_usize.1 {
                for z in start_usize.2..=stop_usize.2 {
                    self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(x + y * self.length + z * self.base_area, vec_index, id);
                }
            }
        }
    }
}

#[derive(Clone, ToBytes, FromBytes, Debug, PartialEq)]
pub enum TurboGridUpdate {
    AddToSet {grid_slot:usize, vec_index:usize, add:usize},
    RemoveFromSet {grid_slot:usize, vec_index:usize, remove:usize},
    MoveFromTo {grid_start:usize, grid_end:usize, vec_index:usize, id:usize}
}



pub struct TGIterator<'a> {
    grid:&'a SparseGrid<TurboSet>,
    current_set:Option<TurboSetIterator<'a>>,
    end_of_line_add:usize,
    end_of_layer_add:usize,
    current_pos:(usize,usize,usize),
    x_limit:usize,
    y_limit:usize,
    z_limit:usize,
    current_index:usize,
    index_in_vec:usize
}

impl<'a> TGIterator<'a> {
    fn update_pos(&mut self) -> bool {
        if self.current_pos.0 == self.x_limit {
            if self.current_pos.1 == self.y_limit {
                if self.current_pos.2 == self.z_limit {
                    /**/
                    false
                }
                else {
                    self.current_pos.0 = 0;
                    self.current_pos.1 = 0;
                    self.current_pos.2 += 1;
                    self.current_index += self.end_of_layer_add;
                    true
                }
            }
            else {
                self.current_pos.0 = 0;
                self.current_pos.1 += 1;
                self.current_index += self.end_of_line_add;
                true
            }
        }
        else {
            self.current_pos.0 += 1;
            self.current_index += 1;
            true
        }
    }
    fn next_slot(&mut self) -> Option<usize> {
        while self.update_pos() {
            if let Some(value) = self.update_iterator() {
                return Some(value)
            }
        }
        if !self.update_pos() {
            match &self.current_set {
                Some(set) => {
                    None
                },
                None => match self.grid.grid[self.current_index] {
                    Some(data_id) => {
                        let set = &self.grid.data[data_id + self.index_in_vec];
                        if set.len() > 0 {
                            let mut new_iterator = set.iter();
                            let value = new_iterator.next().unwrap();
                            self.current_set = Some(new_iterator);
                            Some(value)
                        }
                        else {
                            None
                        }
                    },
                    None => {
                        None
                    }
                }
            }
        }
        else {
            None
        }
    }
    fn update_iterator(&mut self) -> Option<usize> {
        match self.grid.grid[self.current_index] {
            Some(data_id) => {
                let set = &self.grid.data[data_id + self.index_in_vec];
                if set.len() > 0 {
                    let mut new_iterator = set.iter();
                    let value = new_iterator.next().unwrap();
                    self.current_set = Some(new_iterator);
                    Some(value)
                }
                else {
                    None
                }
            },
            None => None
        }
    }
    fn initial_iterator_update(&mut self) {
        match self.grid.grid[self.current_index] {
            Some(data_id) => {
                let new_iterator = self.grid.data[data_id + self.index_in_vec].iter();
                self.current_set = Some(new_iterator);
            },
            None => ()
        }
    }
} 

impl<'a> Iterator for TGIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current_set.as_mut() {
            Some(iterator) => match iterator.next() {
                Some(data) => {
                    Some(data)
                },
                None => self.next_slot(),
            },
            None => self.next_slot(),
        }
    }
}