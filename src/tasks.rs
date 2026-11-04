use task_derive::HordeTask;
use crate::engine::{SinglePEngine, SinglePEngineBase};
use hord3::{defaults::default_rendering::mantle::api::MantleHandler, horde::scheduler::{HordeBackgroundTask, HordeScheduler, HordeTask, HordeTaskData, HordeTaskHandler, IndividualTask}};

#[derive(Clone, PartialEq, Hash, Eq, Debug, HordeTask)]
pub enum TestSinglePlayerTask {
    #[uses_type = "SinglePEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 0]
    ApplyEvents,

    #[uses_type = "SinglePEngineBase"]
    #[max_threads = 3]
    #[type_task_id = 100]
    Main,

    #[uses_type = "SinglePEngineBase"]
    #[max_threads = 3]
    #[type_task_id = 101]
    AfterMain,

    #[uses_type = "SinglePEngineBase"]
    #[max_threads = 1]
    #[type_task_id = 3]
    PrepareRendering,
    
    #[uses_type = "MantleHandler"]
    #[max_threads = 1]
    #[type_task_id = 0]
    DoMantleStuff
}

pub fn nothing() {
    
}