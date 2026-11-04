use crate::{entities::{reclaim::simple_stats::StaticSimpleStats, team::StaticTeam}, shared::{actions::TID, death_accounting::StaticDependSet, resources::StaticTeamResources, simple_position::StaticSimplePos}};

pub fn get_default_team() -> StaticTeam<TID> {
    StaticTeam {
        pos: StaticSimplePos {a:0},
        stats: StaticSimpleStats {upgrade_speed:0.0},
        depend: StaticDependSet {size_hint:256},
        resources: StaticTeamResources {
            nothing:0
        }
    }
}