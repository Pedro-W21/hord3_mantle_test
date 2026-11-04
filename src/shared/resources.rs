use std::ops::{Add, AddAssign, SubAssign};

use hord3::horde::game_engine::{entity::{Component, SimpleComponentEvent, SimpleComponentUpdate, StaticComponent}, multiplayer::Identify};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::shared::consts::RESOURCE_SHARE_SCALE;

#[derive(Clone, Copy, Debug, PartialEq, ToBytes, FromBytes)]
pub struct ResourceCost {
    pub mass:i64,
    pub energy:i64,
    pub server:i64
}

impl ResourceCost {
    pub fn new(mass:i64, energy:i64, server:i64) -> Self {
        Self { mass, energy, server }
    }
}

impl Add<Self> for ResourceCost {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.mass + rhs.mass, self.energy + rhs.energy, self.server + rhs.server)
    }
}

impl Add<&Self> for ResourceCost {
    type Output = Self;
    fn add(self, rhs: &Self) -> Self::Output {
        self + *rhs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, ToBytes, FromBytes)]
pub struct ResourceProduction {
    pub mass:i64,
    pub energy:i64
}
impl ResourceProduction {
    pub fn new(mass:i64, energy:i64 ) -> Self {
        Self { mass, energy }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, ToBytes, FromBytes)]
pub struct ResourceStorage {
    pub mass:i64,
    pub energy:i64,
    pub server:i64
}

impl ResourceStorage {
    pub fn new(mass:i64, energy:i64, server:i64) -> Self {
        Self { mass, energy, server }
    }
}

/// idea :
/// - all consumers add their consumption to total consumption
/// - producers add their production to total production
/// - servers are only capacity based, full on start of tick, taken by various entities
/// 
#[derive(Clone, Copy, Debug, PartialEq, ToBytes, FromBytes)]
pub struct TeamResources {
    mass_storage:i64,
    energy_storage:i64,
    server_storage:i64,
    
    mass_consumption:i64,
    energy_consumption:i64,
    server_consumption:i64,

    mass_production:i64,
    energy_production:i64,

    stored_mass:i64,
    stored_energy:i64
}

impl AddAssign<ResourceCost> for TeamResources {
    fn add_assign(&mut self, rhs: ResourceCost) {
        self.mass_consumption += rhs.mass;
        self.energy_consumption += rhs.energy;
        self.server_consumption += rhs.server;
    }
}

impl SubAssign<ResourceCost> for TeamResources {
    fn sub_assign(&mut self, rhs: ResourceCost) {
        self.mass_consumption -= rhs.mass;
        self.energy_consumption -= rhs.energy;
        self.server_consumption -= rhs.server;
    }
}

impl AddAssign<ResourceProduction> for TeamResources {
    fn add_assign(&mut self, rhs: ResourceProduction) {
        self.mass_production += rhs.mass;
        self.energy_production += rhs.energy;
    }
}

impl AddAssign<ResourceStorage> for TeamResources {
    fn add_assign(&mut self, rhs: ResourceStorage) {
        self.mass_storage += rhs.mass;
        self.energy_storage += rhs.energy;
        self.server_storage += rhs.server;
    }
}

impl SubAssign<ResourceProduction> for TeamResources {
    fn sub_assign(&mut self, rhs: ResourceProduction) {
        self.mass_production -= rhs.mass;
        self.energy_production -= rhs.energy;
    }
}

impl SubAssign<ResourceStorage> for TeamResources {
    fn sub_assign(&mut self, rhs: ResourceStorage) {
        self.mass_storage -= rhs.mass;
        self.energy_storage -= rhs.energy;
        self.server_storage -= rhs.server;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, ToBytes, FromBytes)]
pub enum TeamResourcesUpdate {
    AddConsumption(ResourceCost),
    RemoveConsumption(ResourceCost),
    AddProduction(ResourceProduction),
    AddProdAndStorage(ResourceProduction, ResourceStorage),
    RemoveProduction(ResourceProduction),
    AddStorage(ResourceStorage),
    RemoveStorage(ResourceStorage),
    ZeroOutCosts,
}

#[derive(Clone, Debug)]
pub struct StaticTeamResources {
    pub nothing:u8
}

impl StaticComponent for StaticTeamResources {

}

pub type TeamResourceEvent<ID:Identify> = SimpleComponentEvent<ID, TeamResourcesUpdate>;

impl<ID:Identify> SimpleComponentUpdate<TeamResources, ID> for TeamResourcesUpdate {
    fn apply_to_comp(self, component:&mut TeamResources) {
        match self {
            Self::AddConsumption(cons) => *component += cons,
            Self::RemoveConsumption(cons) => *component -= cons,
            Self::AddProduction(prod) => *component += prod,
            Self::AddStorage(stor) => *component += stor,
            Self::RemoveProduction(prod) => *component -= prod,
            Self::RemoveStorage(stor) => *component -= stor,
            Self::ZeroOutCosts => {
                component.energy_consumption = 0;
                component.mass_consumption = 0;
                component.server_consumption = 0;
            },
            Self::AddProdAndStorage(prod, stor) => {
                *component += prod;
                *component += stor;
            }
        }
    }
}

impl<ID:Identify> Component<ID> for TeamResources {
    type SC = StaticTeamResources;
    type CE = TeamResourceEvent<ID>;
    fn from_static(static_comp:&Self::SC) -> Self {
        Self {
            mass_storage: 1000,
            energy_storage: 1000,
            server_storage: 10,

            mass_consumption: 0,
            energy_consumption: 0,
            server_consumption: 0,

            mass_production: 100,
            energy_production: 1000,

            stored_mass:1000,
            stored_energy:1000
        }
    }
}

impl TeamResources {
    pub fn get_share(&self, cost:ResourceCost) -> i64 {
        let mut energy_share = None;
        if cost.energy > 0 {
            if self.energy_storage > 0 || self.energy_production > self.energy_consumption {
                energy_share = Some(RESOURCE_SHARE_SCALE)
            }
            else {
                energy_share = Some((self.energy_production * RESOURCE_SHARE_SCALE)/self.energy_consumption)
            }
        }
        let mut mass_share = None;
        if cost.mass > 0 {
            if self.mass_storage > 0 || self.mass_production > self.mass_consumption {
                mass_share = Some(RESOURCE_SHARE_SCALE)
            }
            else {
                mass_share = Some((self.mass_production * RESOURCE_SHARE_SCALE)/self.mass_consumption)
            }
        }
        energy_share.unwrap_or(RESOURCE_SHARE_SCALE).min(mass_share.unwrap_or(RESOURCE_SHARE_SCALE))
    }
    pub fn can_server(&self, cost:ResourceCost) -> bool {
        cost.server == 0 || self.server_consumption <= self.server_storage
    }
}

