use hord3::horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify};
use to_from_bytes_derive::{FromBytes, ToBytes};


#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct SimpleHealth {
    pub max_health:i32,
    pub current_health:i32,
}

impl SimpleHealth {
    pub fn is_full(&self) -> bool {
        self.current_health == self.max_health
    }
    pub fn is_dead(&self) -> bool {
        self.current_health <= 0
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StaticSimpleHealth {
    pub base_max_health:i32,
}

impl StaticComponent for StaticSimpleHealth {
    
}

#[derive(Clone, FromBytes, ToBytes, PartialEq)]
pub enum SimpleHealthUpdate {
    Damage(i32),
    Heal(i32)
}

impl Eq for SimpleHealthUpdate {
    
}

impl<ID:Identify> SimpleComponentUpdate<SimpleHealth, ID> for SimpleHealthUpdate {
    fn apply_to_comp(self, component:&mut SimpleHealth) {
        match self {
            SimpleHealthUpdate::Damage(damage) => component.current_health -= damage,
            SimpleHealthUpdate::Heal(heal) => component.current_health = (component.current_health + heal).clamp(0, component.max_health),
        }
    }
}

impl<ID:Identify> Component<ID> for SimpleHealth {
    type CE = SimpleComponentEvent<ID, SimpleHealthUpdate>;
    type SC = StaticSimpleHealth;
    fn from_static(static_comp:&StaticSimpleHealth) -> Self {
        Self {
            max_health:static_comp.base_max_health,
            current_health:static_comp.base_max_health,
        }
    }
}