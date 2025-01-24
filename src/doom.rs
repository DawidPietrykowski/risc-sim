use std::collections::HashMap;

use anyhow::{bail, Result};
use minifb::{Key, Window, WindowOptions};
use once_cell::sync::Lazy;
use risc_sim::cpu::cpu_core::Cpu;

pub const KEY_RIGHTARROW: u8 = 0xae;
pub const KEY_LEFTARROW: u8 = 0xac;
pub const KEY_UPARROW: u8 = 0xad;
pub const KEY_DOWNARROW: u8 = 0xaf;
pub const KEY_STRAFE_L: u8 = 0xa0;
pub const KEY_STRAFE_R: u8 = 0xa1;
pub const KEY_USE: u8 = 0xa2;
pub const KEY_FIRE: u8 = 0xa3;
pub const KEY_ESCAPE: u8 = 27;
pub const KEY_ENTER: u8 = 13;
pub const KEY_TAB: u8 = 9;

const SCREEN_WIDTH: u64 = 320;
const SCREEN_HEIGHT: u64 = 200;
const SCREEN_ADDR_ADDR: u64 = 0x1000000 - 4;
const KEYQUEUE_ADDR_ADDR: u64 = 0x1000000 - 8;
const SCALE_SCREEN: u64 = 6;

static KEY_PAIRS: Lazy<Vec<(Key, u8)>> = Lazy::new(|| {
    vec![
        (Key::W, KEY_UPARROW),
        (Key::S, KEY_DOWNARROW),
        (Key::Left, KEY_LEFTARROW),
        (Key::Right, KEY_RIGHTARROW),
        (Key::Enter, KEY_ENTER),
        (Key::Tab, KEY_TAB),
        (Key::E, KEY_FIRE),
        (Key::Q, KEY_USE),
        (Key::A, KEY_STRAFE_L),
        (Key::D, KEY_STRAFE_R),
        (Key::Escape, KEY_ESCAPE),
    ]
});

pub struct DoomEmulation {
    key_states: HashMap<Key, bool>,
    pub frames_drawn: u64,
    window: Window,
    buffer: Vec<u32>,
}

pub fn doom_init() -> DoomEmulation {
    let mut key_states: HashMap<Key, bool> = HashMap::new();
    for (key, _) in &*KEY_PAIRS {
        key_states.insert(*key, false);
    }

    let mut window = Window::new(
        "DISPLAY",
        (SCALE_SCREEN * SCREEN_WIDTH) as usize,
        (SCALE_SCREEN * SCREEN_HEIGHT) as usize,
        WindowOptions::default(),
    )
    .unwrap();

    let buffer: Vec<u32> = vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT).try_into().unwrap()];

    window
        .update_with_buffer(&buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
        .unwrap();

    DoomEmulation {
        key_states,
        frames_drawn: 0,
        window,
        buffer,
    }
}

pub fn update_window(cpu: &mut Cpu, emulation: &mut DoomEmulation) -> Result<()> {
    if cpu.read_mem_u32(SCREEN_ADDR_ADDR)? == 0 {
        return Ok(());
    }
    emulation.frames_drawn += 1;

    let window = &mut emulation.window;
    let buffer = &mut emulation.buffer;

    if window.is_key_down(Key::Backslash) {
        bail!("Escape pressed");
    }

    let screen_data_addr = cpu.read_mem_u32(SCREEN_ADDR_ADDR)? as u64;

    cpu.write_mem_u32(SCREEN_ADDR_ADDR, 0)?;

    let cmap = false;

    for ypos in 0..SCREEN_HEIGHT {
        for xpos in 0..SCREEN_WIDTH {
            let pixel_index = (xpos) * SCREEN_HEIGHT + (ypos);
            let g;
            let b;
            let r;
            if cmap {
                let val = cpu.read_mem_u8(screen_data_addr + pixel_index)? as u32;
                g = ((val) & 0b11) * 0xFF / 4;
                b = ((val >> 3) & 0b11) * 0xFF / 4;
                r = ((val >> 6) & 0b11) * 0xFF / 4;
            } else {
                let val = cpu.read_mem_u32(screen_data_addr + pixel_index * 4)?;
                r = (val) & 0xFF;
                g = (val >> 8) & 0xFF;
                b = (val >> 16) & 0xFF;
            }
            buffer[pixel_index as usize] = r | (g << 8) | (b << 16) | (0xFF << 24);
        }
    }

    if window.is_open() {
        window
            .update_with_buffer(&buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
            .unwrap();
    }

    let key_states = &mut emulation.key_states;
    let keyqueue_data_addr = cpu.read_mem_u32(KEYQUEUE_ADDR_ADDR)? as u64;
    let keyqueue_data = cpu.read_mem_u32(keyqueue_data_addr)?;
    if keyqueue_data != 0xFFFF_FFFF {
        return Ok(());
    }
    let mut queue_entry_count = 0;
    for (key, doom_key) in &*KEY_PAIRS {
        let down = window.is_key_down(*key);
        let down_prev = key_states[&key];
        let pressed = down && !down_prev;
        let released = !down && down_prev;
        let state = key_states.get_mut(&key).unwrap();
        *state = down;
        if pressed || released {
            cpu.write_mem_u32(
                keyqueue_data_addr + queue_entry_count * 4,
                ((pressed as u32) << 31) | *doom_key as u32,
            )
            .unwrap();
            queue_entry_count += 1;
        }
    }
    cpu.write_mem_u32(keyqueue_data_addr + queue_entry_count * 4, 0xFFFFFFFF)
        .unwrap();

    Ok(())
}
