use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, RwLock};

use crate::vm::{frame::Frame, runtime_class::ClassRef};

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
    frames: Mutex<Vec<Frame>>,
    id: ThreadRef,
    name: String,
    state: RwLock<ThreadState>,
    priority: AtomicUsize,
}

#[allow(dead_code)]
impl Thread {
    pub fn new(id: ThreadRef, name: impl Into<String>) -> Self {
        Self {
            frames: Mutex::new(Vec::new()),
            id,
            name: name.into(),
            state: RwLock::new(ThreadState::New),
            priority: AtomicUsize::new(5),
        }
    }

    pub fn id(&self) -> ThreadRef {
        self.id
    }

    pub fn start(&self) {
        debug_assert_eq!(*self.state.read().unwrap(), ThreadState::New);
        *self.state.write().unwrap() = ThreadState::Runnable;
    }

    pub fn terminate(&self) {
        *self.state.write().unwrap() = ThreadState::Terminated;
    }

    pub fn block(&self) {
        *self.state.write().unwrap() = ThreadState::Blocked;
    }

    pub fn wait(&self) {
        *self.state.write().unwrap() = ThreadState::Waiting;
    }

    pub fn timed_wait(&self) {
        *self.state.write().unwrap() = ThreadState::TimedWaiting;
    }

    pub fn resume_running(&self) {
        *self.state.write().unwrap() = ThreadState::Runnable;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> ThreadState {
        *self.state.read().unwrap()
    }

    pub fn priority(&self) -> usize {
        self.priority.load(Ordering::Relaxed)
    }

    pub fn set_priority(&self, prio: usize) {
        self.priority.store(prio, Ordering::Relaxed);
    }

    pub fn push_frame(&self, frame: Frame) {
        self.frames.lock().unwrap().push(frame);
    }

    pub fn pop_frame(&self) -> Option<Frame> {
        self.frames.lock().unwrap().pop()
    }

    pub fn frame_depth(&self) -> usize {
        self.frames.lock().unwrap().len()
    }

    pub fn lock_frames(&self) -> MutexGuard<'_, Vec<Frame>> {
        self.frames.lock().unwrap()
    }

    pub fn frame_snapshot(&self) -> Vec<(ClassRef, usize)> {
        self.frames
            .lock()
            .unwrap()
            .iter()
            .map(|f| (f.class, f.method_index))
            .collect()
    }
}
