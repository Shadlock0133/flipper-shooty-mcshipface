#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

// global allocator
extern crate alloc;
extern crate flipperzero_alloc;

#[allow(dead_code)]
mod icons;

use core::ffi::CStr;

use bitflags::bitflags;
use flipperzero::{
    furi::{message_queue::MessageQueue, sync::Mutex, time::FuriDuration},
    gui::{
        Gui,
        canvas::{Align, Font},
        view_port::{InputEvent, InputKey, InputType, Orientation, ViewPort},
    },
    println,
};
use flipperzero_rt::{entry, manifest};
use flipperzero_sys as sys;
use tinyvec::ArrayVec;

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
    paused: bool,
    pos: (i32, i32),
    vel_dir: Dir,
    bg_scroll: u8,
    player_bullets: ArrayVec<[(u8, u8); 32]>,
}

bitflags! {
    #[derive(Clone, Copy)]
    struct Dir: u8 {
        const Up = 1 << 0;
        const Down = 1 << 1;
        const Left = 1 << 2;
        const Right = 1 << 3;
    }
}

entry!(main);
fn main(_args: Option<&CStr>) -> i32 {
    println!("Hello, Rust!\r");

    let state = State {
        event_queue: MessageQueue::new(8),
        game_state: Mutex::new(GameState {
            paused: false,
            pos: (32, 120),
            vel_dir: Dir::empty(),
            bg_scroll: 0,
            player_bullets: ArrayVec::new(),
        }),
    };

    let mut view_port = ViewPort::new();
    view_port.set_orientation(Orientation::Vertical);
    view_port.set_draw_callback(|canvas| {
        let game_state = state
            .game_state
            .try_lock_for(FuriDuration::from_secs(1) / 15)
            .unwrap()
            .clone();

        let bg_y = game_state.bg_scroll as i32;
        let far = [(14, 0), (40, 200), (38, 60), (52, 120)];
        for (x, y) in far {
            canvas.draw_dot(x, ((bg_y + y) / 2) % 128);
            canvas.draw_dot(x, (((bg_y + y) / 2) + 1) % 128);
        }
        let near = [(43, 0), (30, 40), (12, 83), (59, 67), (16, 12)];
        for (x, y) in near {
            canvas.draw_dot(x, (bg_y + y) % 128);
            canvas.draw_dot(x, (bg_y + y + 1) % 128);
        }

        for &(x, y) in game_state.player_bullets.iter() {
            canvas.draw_box(x as i32, y as i32, 2, 3);
        }

        let dir = game_state.vel_dir;
        let icon = if dir.contains(Dir::Left) && !dir.contains(Dir::Right) {
            &icons::SHIP_LEFT
        } else if dir.contains(Dir::Right) && !dir.contains(Dir::Left) {
            &icons::SHIP_RIGHT
        } else {
            &icons::SHIP
        };
        canvas.draw_icon(
            game_state.pos.0 - (icon.width / 2) as i32,
            game_state.pos.1 - (icon.height / 2) as i32,
            icon,
        );

        if game_state.paused {
            canvas.set_font(Font::Primary);
            canvas.draw_str_aligned(32, 64, Align::Center, c"Paused");
        }
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
        let event = state.event_queue.get(FuriDuration::from_secs(1) / 30);
        let mut game_state = state.game_state.lock();
        if let Ok(event) = event {
            if event.key == InputKey::Back && event.type_ == InputType::Long {
                break;
            }
            if event.key == InputKey::Back && event.type_ == InputType::Short {
                game_state.paused = !game_state.paused;
            }
            if !game_state.paused {
                if event.type_ == InputType::Press {
                    match event.key {
                        InputKey::Ok => {
                            let (x, y) = game_state.pos;
                            game_state
                                .player_bullets
                                .try_push((x as u8 - 1, y as u8));
                        }
                        InputKey::Up => game_state.vel_dir |= Dir::Up,
                        InputKey::Down => game_state.vel_dir |= Dir::Down,
                        InputKey::Right => game_state.vel_dir |= Dir::Right,
                        InputKey::Left => game_state.vel_dir |= Dir::Left,
                        _ => (),
                    }
                } else if event.type_ == InputType::Release {
                    match event.key {
                        InputKey::Up => game_state.vel_dir &= !Dir::Up,
                        InputKey::Down => game_state.vel_dir &= !Dir::Down,
                        InputKey::Right => game_state.vel_dir &= !Dir::Right,
                        InputKey::Left => game_state.vel_dir &= !Dir::Left,
                        _ => (),
                    }
                }
            }
        }
        if !game_state.paused {
            let dir = game_state.vel_dir;
            let (x, y) = &mut game_state.pos;
            if dir.contains(Dir::Up) {
                *y = (*y - 1).clamp(0, 128)
            }
            if dir.contains(Dir::Down) {
                *y = (*y + 1).clamp(0, 128)
            }
            if dir.contains(Dir::Left) {
                *x = (*x - 1).clamp(0, 64)
            }
            if dir.contains(Dir::Right) {
                *x = (*x + 1).clamp(0, 64)
            }
            game_state.player_bullets.retain_mut(|(_, y)| {
                if *y < 2 {
                    false
                } else {
                    *y -= 2;
                    true
                }
            });
            game_state.bg_scroll = game_state.bg_scroll.wrapping_add(1);
        }
        drop(game_state);
        view_port.update();
    }

    view_port.set_enabled(false);

    0
}
