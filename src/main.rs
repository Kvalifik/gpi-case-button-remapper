mod button_map;

use evdev_rs::{*, enums::*};
use std::{fs::File, io};
use button_map::*;

/*
The GPi case has an option that changes the d-pad mode.
If you hold start+up for 5 seconds, the d-pad changes to UP mode (the defualt d-pad mode).
If you hold start+left for 5 seconds, the d-pad changes to LEFT mode.
(on older models you have to press select instead of start)

In UP mode, the following d-pad signals are sent:

ABS_HAT0X  1 -> hold right
ABS_HAT0X -1 -> hold left
ABS_HAT0X  0 -> release left and right

ABS_HAT0Y  1 -> hold down
ABS_HAT0Y -1 -> hold up
ABS_HAT0Y  0 -> release down and up

In LEFT mode, the d-pad signals are instead

ABS_HAT0X  32767 -> hold right
ABS_HAT0X -32768 -> hold left
ABS_HAT0X  0     -> release left and right

ABS_HAT0Y  32767 -> hold down
ABS_HAT0Y -32768 -> hold up
ABS_HAT0Y -1     -> release down and up

(32767 and -32768 are the maximum and minimum values for a signed 16-bit integer)

Since an ABS_HAT0Y signal can mean two things, we keep track of the current d-pad state
    and try to deduce when it changes.
*/

#[derive(Copy, Clone, Debug)]
enum DPadMode {
    Default,
    Left,
}

// Changes dpad_mode to the correct state if the current dpad mode can
// be deduced from the axis and value.
fn update_dpad_mode(axis: EV_ABS, value: i32, dpad_mode: &mut DPadMode) {
    match (axis, value) {
        (EV_ABS::ABS_HAT0X, 0 | -1 | 1) |
        (EV_ABS::ABS_HAT0Y, 0 | 1) => *dpad_mode = DPadMode::Default,
        (_, v) if v.abs() > 1      => *dpad_mode = DPadMode::Left,
        _ => (),
    }
}

// Simulates a keyboard event by writing an evdev event through a virtal UInput device
fn simulate_keyboard_event(
    virtual_keyboard: &UInputDevice,
    time: &TimeVal,
    code: &EventCode,
    value: i32,
) -> io::Result<()> {
    virtual_keyboard.write_event(&InputEvent::new(
        time,
        code,
        value
    ))?;
    virtual_keyboard.write_event(&InputEvent::new(
        time,
        &EventCode::EV_SYN(EV_SYN::SYN_REPORT),
        0
    ))?;
    Ok(())
}

fn remap_event(
    event: &InputEvent,
    dpad_mode: &mut DPadMode,
    virtual_keyboard: &UInputDevice
) -> io::Result<()> {
    let event_code = event.event_code;
    match event_code {
        // Button press (ABYX, left/right shoulder, select/start)
        EventCode::EV_KEY(ev_key) => {
            if let Some(remap) = BUTTON_MAP.get(&ev_key) {
                simulate_keyboard_event(
                    virtual_keyboard,
                    &event.time,
                    &EventCode::EV_KEY(*remap),
                    event.value
                )?;
            }
        }
        // D-pad
        EventCode::EV_ABS(axis) if axis == EV_ABS::ABS_HAT0X || axis == EV_ABS::ABS_HAT0Y => {
            update_dpad_mode(axis, event.value, dpad_mode);

            // Remap the value signal if the dpad is in left mode
            let value = match (axis, dpad_mode) {
                (EV_ABS::ABS_HAT0X, DPadMode::Left) => event.value.min(1).max(-1),
                (EV_ABS::ABS_HAT0Y, DPadMode::Left) => (event.value + 1).min(1).max(-1),
                (EV_ABS::ABS_HAT0X | EV_ABS::ABS_HAT0Y, DPadMode::Default) => event.value,
                _ => unreachable!(),
            };
            // Send the correct keypress event
            match (axis, value) {
                (EV_ABS::ABS_HAT0X, 1) => {
                    // Hold right
                    simulate_keyboard_event(
                        virtual_keyboard,
                        &event.time,
                        &EventCode::EV_KEY(DPAD_MAP_RIGHT),
                        1
                    )?;
                },
                (EV_ABS::ABS_HAT0X, -1) => {
                    // Hold left
                    simulate_keyboard_event(
                        virtual_keyboard,
                        &event.time,
                        &EventCode::EV_KEY(DPAD_MAP_LEFT),
                        1
                    )?;
                },
                (EV_ABS::ABS_HAT0X, 0) => {
                    // Let go of right and left
                    simulate_keyboard_event(
                        virtual_keyboard,
                        &event.time,
                        &EventCode::EV_KEY(DPAD_MAP_RIGHT),
                        0
                    )?;
                    simulate_keyboard_event(
                        virtual_keyboard,
                        &event.time,
                        &EventCode::EV_KEY(DPAD_MAP_LEFT),
                        0
                    )?;
                },
                (EV_ABS::ABS_HAT0Y, 1) => {
                    // Hold down
                    simulate_keyboard_event(
                        virtual_keyboard,
                        &event.time,
                        &EventCode::EV_KEY(DPAD_MAP_DOWN),
                        1
                    )?;
                },
                (EV_ABS::ABS_HAT0Y, -1) => {
                    // Hold up
                    simulate_keyboard_event(
                        virtual_keyboard,
                        &event.time,
                        &EventCode::EV_KEY(DPAD_MAP_UP),
                        1
                    )?;
                },
                (EV_ABS::ABS_HAT0Y, 0) => {
                    // Let go of down and up
                    simulate_keyboard_event(
                        virtual_keyboard,
                        &event.time,
                        &EventCode::EV_KEY(DPAD_MAP_DOWN),
                        0
                    )?;
                    simulate_keyboard_event(
                        virtual_keyboard,
                        &event.time,
                        &EventCode::EV_KEY(DPAD_MAP_UP),
                        0
                    )?;
                },
                _ => (), // This case should technically be unreachable, but we don't
                         // want to crash the program if we somehow still reach it.
            }
        }
        _ => (),
    }
    Ok(())
}

fn create_virtual_keyboard() -> io::Result<UInputDevice> {
    // https://github.com/ndesh26/evdev-rs/blob/master/examples/vmouse.rs
    let uninit_device = UninitDevice::new().unwrap();
    uninit_device.set_name("Virtual Keyboard");
    uninit_device.set_bustype(BusType::BUS_USB as u16);
    uninit_device.set_vendor_id(0xabcd);
    uninit_device.set_product_id(0xefef);

    uninit_device.enable_event_type(&EventType::EV_KEY)?;

    for key in [DPAD_MAP_LEFT, DPAD_MAP_RIGHT, DPAD_MAP_UP, DPAD_MAP_DOWN] {
        uninit_device.enable_event_code(&EventCode::EV_KEY(key), None)?;
    }
    for key in BUTTON_MAP.values() {
        uninit_device.enable_event_code(&EventCode::EV_KEY(*key), None)?;
    }

    UInputDevice::create_from_device(&uninit_device)
}

fn connect_gamepad(path: &str) -> io::Result<Device> {
    let file = File::open(path)?;
    Device::new_from_file(file)
}

fn main() {
    let gamepad = match connect_gamepad("/dev/input/event0") {
        Ok(gamepad) => gamepad,
        Err(e) => panic!("Error connecting to gamepad: {}", e),
    };
    let virtual_keyboard = match create_virtual_keyboard() {
        Ok(kbd) => kbd,
        Err(e) => panic!("Error creating virtual keyboard: {}", e),
    };

    let mut dpad_mode = DPadMode::Default;
    loop {
        let ev = gamepad.next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING).map(|val| val.1);
        if let Ok(ev) = ev {
            remap_event(&ev, &mut dpad_mode, &virtual_keyboard).unwrap();
        }
    }
}

