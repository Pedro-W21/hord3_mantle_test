use hord3::{defaults::{default_rendering::{mantle::api::MantleHandler, vectorinator_binned::Vectorinator}, default_ui::simple_ui::{SimpleUI, UserEvent}}, horde::{frontend::WindowingHandler, scheduler::{HordeTask, HordeTaskData, HordeTaskHandler, IndividualTask}, sound::ARWWaves}};
use task_derive::HordeTask;

use crate::{game_engine::{CoolGameEngine, CoolGameEngineBase}, turbo_game_engine::TurboGameEngine};

#[derive(Clone, PartialEq, Hash, Eq, Debug, HordeTask)]
pub enum GameTask {
    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 0]
    ApplyEvents,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 8]
    #[type_task_id = 100]
    Stage0,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 8]
    #[type_task_id = 101]
    Stage1,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 8]
    #[type_task_id = 102]
    Stage2,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 8]
    #[type_task_id = 103]
    Stage3,

    #[uses_type = "CoolGameEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 3]
    PrepareRendering,
    
    #[uses_type = "MantleHandler"]
    #[max_threads = 1]
    #[type_task_id = 0]
    DoMantleStuff,

    #[uses_type = "ARWWaves"]
    #[uses_generic = "CoolGameEngine"]
    #[max_threads = 1]
    #[type_task_id = 0]
    UpdateSoundPositions,

    #[uses_type = "ARWWaves"]
    #[uses_generic = "CoolGameEngine"]
    #[max_threads = 1]
    #[type_task_id = 1]
    UpdateSoundEverythingElse,
    
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum GameUserEvent {
    ClickedCoolButton,
    ClickedBadButton,
    IncreasedThatValue(String),
    DecreasedThatValue(String),
    ChoseThatValue(String, String)
}

impl UserEvent for GameUserEvent {

}

fn cool() {
    
}