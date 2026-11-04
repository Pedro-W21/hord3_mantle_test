use hord3::horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify};
use to_from_bytes_derive::{FromBytes, ToBytes};


#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct Health {
    pub max_health:i32,
    pub current_health:i32,
    effects:Vec<StatusEffect>
}

impl Health {
    pub fn is_full(&self) -> bool {
        self.current_health >= self.max_health
    }
    pub fn is_dead(&self) -> bool {
        self.current_health <= 0
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub struct StatusEffect {
    time_left:usize,
    effect_type:StatusEffectType
}

#[derive(Clone, ToBytes, FromBytes, PartialEq, Debug)]
pub enum StatusEffectType {
    Poison,
    Healing
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct StaticHealth {
    pub base_max_health:i32,
    pub starting_effects:Vec<StatusEffect>
}

impl StaticComponent for StaticHealth {
    
}

#[derive(Clone, FromBytes, ToBytes, PartialEq)]
pub enum HealthUpdate {
    Damage(i32),
    Heal(i32)
}

impl Eq for HealthUpdate {
    
}

impl<ID:Identify> SimpleComponentUpdate<Health, ID> for HealthUpdate {
    fn apply_to_comp(self, component:&mut Health) {
        match self {
            HealthUpdate::Damage(damage) => component.current_health -= damage,
            HealthUpdate::Heal(heal) => component.current_health = (component.current_health + heal).clamp(0, component.max_health),
        }
    }
}

impl<ID:Identify> Component<ID> for Health {
    type CE = SimpleComponentEvent<ID, HealthUpdate>;
    type SC = StaticHealth;
    fn from_static(static_comp:&StaticHealth) -> Self {
        Self {
            max_health:static_comp.base_max_health,
            current_health:static_comp.base_max_health,
            effects:static_comp.starting_effects.clone()
        }
    }
}