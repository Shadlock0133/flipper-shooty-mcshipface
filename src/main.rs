#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

// global allocator
extern crate alloc;
extern crate flipperzero_alloc;

mod icons;

use core::ffi::CStr;

use flipperzero::{
    furi::{message_queue::MessageQueue, sync::Mutex, time::FuriDuration},
    gui::{
        Gui,
        view_port::{InputEvent, InputKey, InputType, Orientation, ViewPort},
    },
    println,
};
use flipperzero_rt::{entry, manifest};
use flipperzero_sys as sys;

manifest!(
    name = "Shooty McShipface",
    app_version = 1,
    has_icon = true,
    icon = concat!(env!("OUT_DIR"), "/icon.icon"),
);

struct State {
    event_queue: MessageQueue<InputEvent>,
    game_state: Mutex<GameState>,
}

#[derive(Clone)]
struct GameState {
    pos: (i32, i32),
    vel_dir: Option<Dir>,
}

#[derive(Clone, Copy)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

entry!(main);
fn main(_args: Option<&CStr>) -> i32 {
    println!("Hello, Rust!\r");

    let state = State {
        event_queue: MessageQueue::new(8),
        game_state: Mutex::new(GameState {
            pos: (32, 120),
            vel_dir: None,
        }),
    };

    let mut view_port = ViewPort::new();
    view_port.set_orientation(Orientation::Vertical);
    view_port.set_draw_callback(|canvas| {
        let game_state = state.game_state.lock().clone();
        canvas.draw_icon(
            game_state.pos.0 - (icons::SHIP.width / 2) as i32,
            game_state.pos.1 - (icons::SHIP.height / 2) as i32,
            &icons::SHIP,
        );
    });
    view_port.set_input_callback(|input| {
        state
            .event_queue
            .put(input, FuriDuration::WAIT_FOREVER)
            .unwrap();
    });

    let gui = Gui::open();
    let view_port = gui.add_view_port(view_port, sys::GuiLayerFullscreen);

    loop {
        let mut game_state = state.game_state.lock();
        if let Ok(event) =
            state.event_queue.get(FuriDuration::from_secs(1) / 30)
        {
            if event.key == InputKey::Back && event.type_ == InputType::Long {
                break;
            }
            if event.type_ == InputType::Press {
                match event.key {
                    InputKey::Up => game_state.vel_dir = Some(Dir::Up),
                    InputKey::Down => game_state.vel_dir = Some(Dir::Down),
                    InputKey::Right => game_state.vel_dir = Some(Dir::Right),
                    InputKey::Left => game_state.vel_dir = Some(Dir::Left),
                    _ => (),
                }
            } else if event.type_ == InputType::Release
                && [
                    InputKey::Up,
                    InputKey::Down,
                    InputKey::Left,
                    InputKey::Right,
                ]
                .contains(&event.key)
            {
                game_state.vel_dir = None
            }
        }
        view_port.update();
    }

    view_port.set_enabled(false);

    0
}
