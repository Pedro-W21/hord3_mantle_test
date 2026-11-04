use std::collections::HashSet;

use hord3::horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify};
use to_from_bytes_derive::{FromBytes, ToBytes};


#[derive(Clone, PartialEq, Eq, ToBytes, FromBytes)]
pub struct DependSet<ID:Identify> {
    dependents: HashSet<ID>
}

impl<ID:Identify> DependSet<ID> {
    pub fn get_dependents(&self) -> &HashSet<ID> {
        &self.dependents
    }
}

#[derive(Clone)]
pub struct StaticDependSet {
    pub size_hint:usize
}

impl StaticComponent for StaticDependSet {

}

impl<ID:Identify, ID2:Identify> Component<ID> for DependSet<ID2> {
    type CE = DependSetEvent<ID, ID2>;
    type SC = StaticDependSet;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self { dependents: HashSet::with_capacity(static_comp.size_hint) }
    }
}

#[derive(Clone, FromBytes, ToBytes, PartialEq, Eq)]
pub enum DependSetUpdate<ID:Identify> {
    Depend(ID),
    StopDepend(ID)
}

impl<ID:Identify, ID2:Identify> SimpleComponentUpdate<DependSet<ID2>, ID> for DependSetUpdate<ID2> {
    fn apply_to_comp(self, component:&mut DependSet<ID2>) {
        match self {
            Self::Depend(id) => component.dependents.insert(id),
            Self::StopDepend(id) => component.dependents.remove(&id)
        };
    }
}

pub type DependSetEvent<ID:Identify, ID2:Identify> = SimpleComponentEvent<ID, DependSetUpdate<ID2>>;