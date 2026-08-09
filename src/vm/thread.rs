use crate::vm::frame::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThreadRef(pub usize);

#[allow(dead_code)]
pub struct Thread {
    pub frames: Vec<Frame>,
    pub id: ThreadRef,
    pub name: String,
}

#[allow(dead_code)]
impl Thread {
    pub fn new(id: ThreadRef, name: impl Into<String>) -> Self {
        Self {
            frames: Vec::new(),
            id: id,
            name: name.into(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn frames(&self) -> &Vec<Frame> {
        &self.frames
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
