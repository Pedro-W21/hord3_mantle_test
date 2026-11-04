use hord3::horde::{game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify, position::EntityPosition}, geometry::{rotation::Orientation, vec3d::Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};


#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct SimplePos {
    pub pos:Vec3Df
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StaticSimplePos {
    pub a:u8
}

impl StaticComponent for StaticSimplePos {
    
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum SimplePosUpdate {
    Update(Vec3Df)
}

impl Eq for SimplePosUpdate {
    
}

impl<ID:Identify> SimpleComponentUpdate<SimplePos, ID> for SimplePosUpdate {
    fn apply_to_comp(self, component:&mut SimplePos) {
        match self {
            SimplePosUpdate::Update(pos) => component.pos = pos,
        }
    }
}

impl<ID:Identify> Component<ID> for SimplePos {
    type CE = SimpleComponentEvent<ID, SimplePosUpdate>;
    type SC = StaticSimplePos;
    fn from_static(static_comp:&StaticSimplePos) -> Self {
        Self {
            pos:Vec3Df::zero()
        }
    }
}

impl<ID:Identify> EntityPosition<ID> for SimplePos {
    fn get_orientation(&self) -> hord3::horde::geometry::rotation::Orientation {
        Orientation::zero()
    }
    fn get_pos(&self) -> Vec3Df {
        self.pos
    }
    fn get_rotation(&self) -> Option<&hord3::horde::geometry::rotation::Rotation> {
        None
    }
}