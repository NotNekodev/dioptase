use crate::vm::frame::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadRef(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    New,
    Runnable,
    Blocked,
    Waiting,
    TimedWaiting,
    Terminated,
}

#[allow(dead_code)]
pub struct Thread {
    frames: Vec<Frame>,
    id: ThreadRef,
    name: String,
    state: ThreadState,
    priority: usize,
}

#[allow(dead_code)]
impl Thread {
    pub fn new(id: ThreadRef, name: impl Into<String>) -> Self {
        Self {
            frames: Vec::new(),
            id: id,
            name: name.into(),
            state: ThreadState::New,
            priority: 5,
        }
    }

    pub fn start(&mut self) {
        debug_assert_eq!(self.state, ThreadState::New);
        self.state = ThreadState::Runnable;
    }

    pub fn terminate(&mut self) {
        self.state = ThreadState::Terminated;
    }

    pub fn block(&mut self) {
        self.state = ThreadState::Blocked;
    }

    pub fn wait(&mut self) {
        self.state = ThreadState::Waiting
    }

    pub fn timed_wait(&mut self) {
        self.state = ThreadState::TimedWaiting;
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn frames(&self) -> &Vec<Frame> {
        &self.frames
    }

    pub fn state(&self) -> &ThreadState {
        &self.state
    }

    pub fn priority(&self) -> usize {
        self.priority
    }

    pub fn set_priority(&mut self, prio: usize) {
        self.priority = prio;
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<Frame> {
        self.frames.pop()
    }

    pub fn current_frame(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }
}
