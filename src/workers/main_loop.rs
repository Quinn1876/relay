pub trait MainLoop<T> {
    fn main_loop(self) -> T;
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

impl<S,R,C,D>
WorkerState<S,R,C,D>
where   S: MainLoop<Self>,
        R: MainLoop<Self>,
        C: MainLoop<Self>,
        D: MainLoop<Self>
{
    pub fn main_loop(self) -> Self {
        match self {
            WorkerState::Startup(worker) => worker.main_loop(),
            WorkerState::Disconnected(worker) => worker.main_loop(),
            WorkerState::Connected(worker) => worker.main_loop(),
            WorkerState::Recovery(worker) => worker.main_loop(),
        }
    }
}
