pub mod drawing_state;
pub mod selection_state;
pub mod ui_state;

pub use drawing_state::DrawingState;
pub use selection_state::SelectionState;
pub use ui_state::UiState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Drawing,
    View,
}
