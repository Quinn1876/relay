/**
 * Each Worker Implements MainLoop.
 * Each worker has an associated worker state which it will give as an argument to MainLoop.
 * The manager of the worker will then initialize the worker and it's state object in the following manner::
 * #[repr(C)]
 * struct Worker<State = Startup> {
 *  worker_data: u8,
 *  state: std::marker::PhantomData<State>
 * }
 *
 * pub type WorkerWorkerState = WorkerState<Startup, Recovery, Connected, Disconnected>;
 *
 * impl MainLoop<WorkerWorkerState> for WorkerState<Startup> {
 *  fn main_loop(self) -> WorkerWorkerState {
 *      // Do stuff here
 *      WorkerWorkerState::Startup(self)
 * }
 * }
 * impl MainLoop<WorkerWorkerState> for WorkerState<Recovery>  {
 *    fn main_loop(self) -> WorkerWorkerState {
 *      // Do stuff here
 *      WorkerWorkerState::Recovery(self)
 *    }
 * }
 * impl MainLoop<WorkerWorkerState> for WorkerState<Connected> {
 *      fn main_loop(self) -> WorkerWorkerState {
 *          // Do stuff here
 *          WorkerWorkerState::Connected(self)
 *      }
 * }
 * impl MainLoop<WorkerWorkerState> for WorkerState<Disconnected> {
 *    fn main_loop(self) -> WorkerWorkerState {
 *      // Do stuff here
 *      WorkerWorkerState::Disconnected(self)
 *    }
 * }
 *
 *
 * NOTE: Where is says "Do stuff here" It is helpful to also implement functions which transform a worker from One state to another. This can be done quickly with the help of std::mem::transmute
 */
pub trait MainLoop<T>
where T: WorkerStateTrait {
    fn main_loop(self) -> T;
}

pub trait WorkerStateTrait {
    fn main_loop(self) -> Self;
}

pub enum WorkerState<S,R,C,D>
where S: MainLoop<Self>,
R: MainLoop<Self>,
C: MainLoop<Self>,
D: MainLoop<Self> {
    Startup(S),
    Recovery(R),
    Connected(C),
    Disconnected(D)
}

impl<S,R,C,D> WorkerStateTrait for
WorkerState<S,R,C,D>
where   S: MainLoop<Self>,
        R: MainLoop<Self>,
        C: MainLoop<Self>,
        D: MainLoop<Self>
{
    fn main_loop(self) -> Self {
        match self {
            WorkerState::Startup(worker) => worker.main_loop(),
            WorkerState::Disconnected(worker) => worker.main_loop(),
            WorkerState::Connected(worker) => worker.main_loop(),
            WorkerState::Recovery(worker) => worker.main_loop(),
        }
    }
}
