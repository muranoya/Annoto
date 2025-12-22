use crate::state::TouchPoint;
use egui;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::TouchEvent;

pub struct TouchState {
    pub current_touches: Vec<TouchPoint>,
    pub previous_touches: Vec<TouchPoint>,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            current_touches: Vec::new(),
            previous_touches: Vec::new(),
        }
    }
}

static TOUCH_STATE: Lazy<Arc<Mutex<TouchState>>> =
    Lazy::new(|| Arc::new(Mutex::new(TouchState::default())));

pub fn init_touch_handlers() {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    // document 全体にタッチイベントリスナーを登録
    let doc_element = document.document_element().expect("no document element");

    // touchstart イベント
    let closure = Closure::wrap(Box::new(|event: TouchEvent| {
        handle_touch_start(&event);
    }) as Box<dyn FnMut(TouchEvent)>);
    doc_element
        .add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();

    // touchmove イベント
    let closure = Closure::wrap(Box::new(|event: TouchEvent| {
        handle_touch_move(&event);
    }) as Box<dyn FnMut(TouchEvent)>);
    doc_element
        .add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();

    // touchend イベント
    let closure = Closure::wrap(Box::new(|event: TouchEvent| {
        handle_touch_end(&event);
    }) as Box<dyn FnMut(TouchEvent)>);
    doc_element
        .add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();
}

fn handle_touch_start(event: &TouchEvent) {
    let mut state = TOUCH_STATE.lock().unwrap();
    state.previous_touches = state.current_touches.clone();
    state.current_touches.clear();

    let touches = event.touches();
    for i in 0..touches.length() {
        if let Some(touch) = touches.get(i) {
            let touch_point = TouchPoint {
                pos: egui::Pos2::new(touch.client_x() as f32, touch.client_y() as f32),
                id: touch.identifier() as u64,
            };
            state.current_touches.push(touch_point);
        }
    }

    web_sys::console::log_1(
        &format!("Touch start: {} touches", state.current_touches.len()).into(),
    );
}

fn handle_touch_move(event: &TouchEvent) {
    let mut state = TOUCH_STATE.lock().unwrap();
    state.previous_touches = state.current_touches.clone();
    state.current_touches.clear();

    let touches = event.touches();
    for i in 0..touches.length() {
        if let Some(touch) = touches.get(i) {
            let touch_point = TouchPoint {
                pos: egui::Pos2::new(touch.client_x() as f32, touch.client_y() as f32),
                id: touch.identifier() as u64,
            };
            state.current_touches.push(touch_point);
        }
    }

    if state.current_touches.len() >= 2 {
        web_sys::console::log_1(
            &format!("Touch move: {} touches", state.current_touches.len()).into(),
        );
    }
}

fn handle_touch_end(event: &TouchEvent) {
    let mut state = TOUCH_STATE.lock().unwrap();
    state.previous_touches = state.current_touches.clone();
    state.current_touches.clear();

    let touches = event.touches();
    for i in 0..touches.length() {
        if let Some(touch) = touches.get(i) {
            let touch_point = TouchPoint {
                pos: egui::Pos2::new(touch.client_x() as f32, touch.client_y() as f32),
                id: touch.identifier() as u64,
            };
            state.current_touches.push(touch_point);
        }
    }
}

pub fn get_current_touches() -> Vec<TouchPoint> {
    TOUCH_STATE.lock().unwrap().current_touches.clone()
}
