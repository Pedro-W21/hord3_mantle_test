use hord3::horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify, static_type_id::HasStaticTypeID};
use to_from_bytes_derive::{FromBytes, ToBytes};

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub struct SimpleStats {
    pub team:u16,
    pub dead:bool,
    static_type_id:usize,
}

impl SimpleStats {
    pub fn new(team:u16, static_type_id:usize) -> Self {
        Self { team, dead: false, static_type_id }
    }
}

impl HasStaticTypeID for SimpleStats {
    fn get_id(&self) -> usize {
        self.static_type_id
    }
}

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub enum SimpleStatsUpdate {
    ChangeTeam(u16),
    Kill
}

impl<ID:Identify> SimpleComponentUpdate<SimpleStats, ID> for SimpleStatsUpdate {
    fn apply_to_comp(self, component:&mut SimpleStats) {
        match self {
            Self::ChangeTeam(new_team) => component.team = new_team,
            Self::Kill => component.dead = true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, ToBytes, FromBytes)]
pub struct StaticSimpleStats {
    pub upgrade_speed:f32,
}

impl StaticComponent for StaticSimpleStats {

}

pub type SimpleStatsEvent<ID:Identify> = SimpleComponentEvent<ID, SimpleStatsUpdate>;

impl<ID:Identify> Component<ID> for SimpleStats {
    type CE = SimpleComponentEvent<ID, SimpleStatsUpdate>;
    type SC = StaticSimpleStats;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { team: 0, static_type_id: 0, dead:false }
    }
}


