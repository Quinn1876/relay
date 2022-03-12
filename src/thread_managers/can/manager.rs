use super::worker::{ CanWorkerState, CanWorkerInitializer };
use super::super::main_loop::WorkerStateTrait;
pub struct CanManager {
}

impl CanManager {
    pub fn run(
        initializer: CanWorkerInitializer
    ) -> std::thread::JoinHandle<()> {
        std::thread::Builder::new().name("CAN Thread".to_string()).spawn(move || {
            // Setup
            let mut can_worker = CanWorkerState::new(initializer);
            loop {
                can_worker = can_worker.main_loop();
            }
        }).expect("Should be able to create Thread")
    }
}
