use crate::vm::frame::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadRef(pub usize);

#[allow(dead_code)]
pub struct Thread {
    pub frames: Vec<Frame>,
    pub id: usize,
}

#[allow(dead_code)]
impl Thread {
    pub fn new(id: usize) -> Self {
        Self {
            frames: Vec::new(),
            id: id,
        }
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
