use hord3::horde::{game_engine::{entity::{Component, ComponentEvent, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify, position::EntityPosition}, geometry::{rotation::Orientation, vec3d::Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::game_map::VoxelGetHint;

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct Position {
    pub pos:Vec3Df,
    pub spd:Vec3Df,
    pub orientation:Orientation,
    pub turn_spd:Orientation,
    pub against_wall:bool,
    pub touching_ground:bool,
    pub voxel_hint:Option<VoxelGetHint>
}
impl Position {
    pub fn new() -> Self {
        Self { pos: Vec3Df::zero(), spd: Vec3Df::zero(), orientation: Orientation::zero(), turn_spd: Orientation::zero(), against_wall:false, touching_ground:false, voxel_hint:None }
    }
    pub fn with_pos(mut self, pos:Vec3Df) -> Self {
        self.pos = pos;
        self
    }
    pub fn with_speed(mut self, spd:Vec3Df) -> Self {
        self.spd = spd;
        self
    }
}
#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StaticPosition {
    pub ground_speed:f32,
    pub jump_height:f32,
    pub static_entity:bool
}

impl StaticComponent for StaticPosition {

}

impl<ID:Identify> Component<ID> for Position {
    type SC = StaticPosition;
    type CE = SimpleComponentEvent<ID, PositionUpdate>;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { pos: Vec3Df::zero(), spd: Vec3Df::zero(), orientation: Orientation::zero(), turn_spd: Orientation::zero(), against_wall:false, touching_ground:false, voxel_hint:None }
    }
}
#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum PositionUpdate {
    UpdateEverySpeed(Vec3Df, Orientation),
    AddToEverySpeed(Vec3Df, Orientation),
    UpdateEveryPos(Vec3Df, Orientation),
    AddToSpeed(Vec3Df),
    UpdateMoveSpeed(Vec3Df),
    UpdatePos(Vec3Df),
    UpdatePosAndHint(Vec3Df, Option<VoxelGetHint>),
    UpdatePosAndMoveSpeedAndHint(Vec3Df, Vec3Df, Option<VoxelGetHint>),
    UpdateTouchingGround(bool),
    UpdateAgainstWall(bool),
    UpdateAgainstBoth {ground:bool, wall:bool}
}

impl<ID:Identify> SimpleComponentUpdate<Position, ID> for PositionUpdate {
    fn apply_to_comp(self, component:&mut Position) {
        match self {
            PositionUpdate::UpdatePos(pos) => {
                component.pos = pos;
            },
            PositionUpdate::UpdateMoveSpeed(spd) => {
                component.spd = spd;
            },
            PositionUpdate::UpdateEverySpeed(spd, turn_spd) => {
                component.spd = spd;
                component.turn_spd = turn_spd;
            },
            PositionUpdate::AddToEverySpeed(spd, turn_spd) => {
                component.spd += spd;
                component.turn_spd += turn_spd;
            },
            PositionUpdate::UpdateEveryPos(pos, orientation) => {
                component.pos = pos;
                component.orientation = orientation;
            },
            PositionUpdate::UpdatePosAndHint(pos, hint) => {
                component.pos = pos;
                component.voxel_hint = hint;
            },
            PositionUpdate::UpdatePosAndMoveSpeedAndHint(pos, spd, hint) => {
                component.pos = pos;
                component.spd = spd;
                component.voxel_hint = hint;

            },
            PositionUpdate::AddToSpeed(spd) => {
                component.spd += spd;
            },
            PositionUpdate::UpdateTouchingGround(ground) => {
                component.touching_ground = ground;
            },
            PositionUpdate::UpdateAgainstWall(wall) => {
                component.against_wall = wall;
            },
            PositionUpdate::UpdateAgainstBoth { ground, wall } => {
                component.touching_ground = ground;
                component.against_wall = wall;
            }
        }
    }
}

pub type PositionEvent<ID:Identify> = SimpleComponentEvent<ID, PositionUpdate>;

impl<ID:Identify> EntityPosition<ID> for Position {
    fn get_orientation(&self) -> Orientation {
        self.orientation
    }
    fn get_pos(&self) -> Vec3Df {
        self.pos
    }
    fn get_rotation(&self) -> Option<&hord3::horde::geometry::rotation::Rotation> {
        None
    }
}
