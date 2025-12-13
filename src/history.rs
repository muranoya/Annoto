use crate::canvas_items::CanvasItem;

#[derive(Clone)]
pub struct History {
    past: Vec<Vec<CanvasItem>>,
    future: Vec<Vec<CanvasItem>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
        }
    }

    pub fn push(&mut self, state: Vec<CanvasItem>) {
        self.past.push(state);
        self.future.clear();
    }

    pub fn undo(&mut self) -> Option<Vec<CanvasItem>> {
        self.past.pop().map(|state| {
            self.future.push(state.clone());
            state
        })
    }

    pub fn redo(&mut self) -> Option<Vec<CanvasItem>> {
        self.future.pop().map(|state| {
            self.past.push(state.clone());
            state
        })
    }
}
