use crate::canvas_items::Handle;

pub struct SelectionState {
    pub selected_item: Option<usize>,
    pub selected_handle: Option<Handle>,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected_item: None,
            selected_handle: None,
        }
    }
}
