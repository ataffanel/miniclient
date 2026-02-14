use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crazyflie_lib::{Crazyflie, NoTocCache};
use crazyflie_link::LinkContext;
use gilrs::{Axis, EventType, Gilrs};
use tokio_util::sync::CancellationToken;

slint::include_modules!();

const MAX_ROLL: f32 = 30.0; // degrees
const MAX_PITCH: f32 = 30.0; // degrees
const MAX_YAWRATE: f32 = 200.0; // degrees/second
const MAX_THRUST: u16 = 60000;
const DEADZONE: f32 = 0.1;

#[derive(Clone, Default)]
struct GamepadAxes {
    roll: f32,
    pitch: f32,
    yawrate: f32,
    thrust: u16,
}

fn apply_deadzone(value: f32) -> f32 {
    if value.abs() < DEADZONE {
        0.0
    } else {
        // Rescale so that the range just outside deadzone starts at 0
        let sign = value.signum();
        sign * (value.abs() - DEADZONE) / (1.0 - DEADZONE)
    }
}

fn update_status(ui: &slint::Weak<MainWindow>, text: &str) {
    let text = text.to_string();
    let ui = ui.clone();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_status_text(text.into());
        }
    })
    .ok();
}

fn set_connected(ui: &slint::Weak<MainWindow>, connected: bool) {
    let ui = ui.clone();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_connected(connected);
        }
    })
    .ok();
}

fn update_state_estimate(
    ui: &slint::Weak<MainWindow>,
    roll: f32,
    pitch: f32,
    yaw: f32,
) {
    let ui = ui.clone();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_roll_state(roll);
            ui.set_pitch_state(pitch);
            ui.set_yaw_state(yaw);
        }
    })
    .ok();
}

fn update_setpoints(ui: &slint::Weak<MainWindow>, axes: &GamepadAxes) {
    let roll = axes.roll;
    let pitch = axes.pitch;
    let yawrate = axes.yawrate;
    let ui = ui.clone();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_roll_setpoint(roll);
            ui.set_pitch_setpoint(pitch);
            ui.set_yaw_setpoint(yawrate);
        }
    })
    .ok();
}

fn update_gamepad_list(ui: &slint::Weak<MainWindow>, names: Vec<String>) {
    let ui = ui.clone();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            let model: Vec<slint::SharedString> =
                names.iter().map(|s| s.into()).collect();
            ui.set_gamepad_list(slint::ModelRc::from(model.as_slice()));
            ui.set_gamepad_index(0);
        }
    })
    .ok();
}

/// Spawns a dedicated thread for gamepad polling (gilrs is not Send).
/// Returns the shared axes state that the connection task will read.
fn spawn_gamepad_thread(
    ui: slint::Weak<MainWindow>,
    selected_index: Arc<AtomicUsize>,
) -> Arc<Mutex<GamepadAxes>> {
    let axes = Arc::new(Mutex::new(GamepadAxes::default()));
    let axes_clone = axes.clone();

    std::thread::spawn(move || {
        let mut gilrs = match Gilrs::new() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Failed to initialize gilrs: {e}");
                return;
            }
        };

        // Collect initially connected gamepads
        let mut gamepad_ids: Vec<gilrs::GamepadId> = Vec::new();
        let mut names: Vec<String> = Vec::new();
        for (id, gamepad) in gilrs.gamepads() {
            gamepad_ids.push(id);
            names.push(gamepad.name().to_string());
        }

        if names.is_empty() {
            update_gamepad_list(&ui, vec!["No gamepad detected".into()]);
        } else {
            update_gamepad_list(&ui, names);
        }

        loop {
            // Process events (connect/disconnect)
            while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
                match event {
                    EventType::Connected => {
                        let name = gilrs.gamepad(id).name().to_string();
                        if !gamepad_ids.contains(&id) {
                            gamepad_ids.push(id);
                        }
                        let names: Vec<String> = gamepad_ids
                            .iter()
                            .map(|gid| gilrs.gamepad(*gid).name().to_string())
                            .collect();
                        update_gamepad_list(&ui, names);
                        eprintln!("Gamepad connected: {name}");
                    }
                    EventType::Disconnected => {
                        gamepad_ids.retain(|gid| *gid != id);
                        if gamepad_ids.is_empty() {
                            update_gamepad_list(&ui, vec!["No gamepad detected".into()]);
                        } else {
                            let names: Vec<String> = gamepad_ids
                                .iter()
                                .map(|gid| gilrs.gamepad(*gid).name().to_string())
                                .collect();
                            update_gamepad_list(&ui, names);
                        }
                        eprintln!("Gamepad disconnected");
                    }
                    _ => {}
                }
            }

            // Read axes from the selected gamepad
            let idx = selected_index.load(Ordering::Relaxed);
            if let Some(&gid) = gamepad_ids.get(idx) {
                let gp = gilrs.gamepad(gid);

                let left_x = apply_deadzone(gp.value(Axis::LeftStickX));
                let left_y = apply_deadzone(gp.value(Axis::LeftStickY));
                let right_x = apply_deadzone(gp.value(Axis::RightStickX));
                let right_y = apply_deadzone(gp.value(Axis::RightStickY));

                // Map axes to flight controls
                let roll = left_x * MAX_ROLL;
                let pitch = left_y * MAX_PITCH; // gilrs: +Y = up = forward
                let yawrate = right_x * MAX_YAWRATE;
                // Thrust: center = 0, only above center gives thrust
                let thrust_f = right_y.max(0.0);
                let thrust = (thrust_f * MAX_THRUST as f32) as u16;

                *axes_clone.lock().unwrap() = GamepadAxes {
                    roll,
                    pitch,
                    yawrate,
                    thrust,
                };
            } else {
                *axes_clone.lock().unwrap() = GamepadAxes::default();
            }

            std::thread::sleep(Duration::from_millis(10)); // 100Hz polling
        }
    });

    axes
}

async fn run_connection(
    ui: slint::Weak<MainWindow>,
    link_context: Arc<LinkContext>,
    uri: String,
    cancel: CancellationToken,
    gamepad_axes: Arc<Mutex<GamepadAxes>>,
    gamepad_enabled: Arc<AtomicBool>,
) {
    update_status(&ui, "Connecting...");

    let cf = match Crazyflie::connect_from_uri(&link_context, &uri, NoTocCache).await {
        Ok(cf) => cf,
        Err(e) => {
            update_status(&ui, &format!("Error: {e}"));
            set_connected(&ui, false);
            return;
        }
    };

    set_connected(&ui, true);
    update_status(&ui, "Connected - setting up logging...");

    // Create log block for state estimation
    let mut block = match cf.log.create_block().await {
        Ok(b) => b,
        Err(e) => {
            update_status(&ui, &format!("Log error: {e}"));
            cf.disconnect().await;
            set_connected(&ui, false);
            return;
        }
    };

    let variables = [
        "stateEstimate.roll",
        "stateEstimate.pitch",
        "stateEstimate.yaw",
    ];
    for var in &variables {
        if let Err(e) = block.add_variable(var).await {
            update_status(&ui, &format!("Log var error ({var}): {e}"));
            cf.disconnect().await;
            set_connected(&ui, false);
            return;
        }
    }

    let period = match Duration::from_millis(100).try_into() {
        Ok(p) => p,
        Err(e) => {
            update_status(&ui, &format!("Period error: {e}"));
            cf.disconnect().await;
            set_connected(&ui, false);
            return;
        }
    };

    let stream = match block.start(period).await {
        Ok(s) => s,
        Err(e) => {
            update_status(&ui, &format!("Log start error: {e}"));
            cf.disconnect().await;
            set_connected(&ui, false);
            return;
        }
    };

    // Unlock thrust by sending a zero setpoint
    if let Err(e) = cf.commander.setpoint_rpyt(0.0, 0.0, 0.0, 0).await {
        update_status(&ui, &format!("Commander unlock error: {e}"));
        cf.disconnect().await;
        set_connected(&ui, false);
        return;
    }

    update_status(&ui, "Connected");

    // Commander interval: send setpoints at 50Hz
    let mut cmd_interval = tokio::time::interval(Duration::from_millis(20));

    // Main loop: read logs + send commander setpoints
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break;
            }
            result = stream.next() => {
                match result {
                    Ok(data) => {
                        let roll = data.data.get("stateEstimate.roll")
                            .map(|v| v.to_f64_lossy() as f32).unwrap_or(0.0);
                        let pitch = data.data.get("stateEstimate.pitch")
                            .map(|v| v.to_f64_lossy() as f32).unwrap_or(0.0);
                        let yaw = data.data.get("stateEstimate.yaw")
                            .map(|v| v.to_f64_lossy() as f32).unwrap_or(0.0);
                        update_state_estimate(&ui, roll, pitch, yaw);
                    }
                    Err(e) => {
                        update_status(&ui, &format!("Log error: {e}"));
                        break;
                    }
                }
            }
            _ = cmd_interval.tick() => {
                if gamepad_enabled.load(Ordering::Relaxed) {
                    let axes = gamepad_axes.lock().unwrap().clone();
                    // Update UI setpoint display with local gamepad values
                    update_setpoints(&ui, &axes);
                    if let Err(e) = cf.commander.setpoint_rpyt(
                        axes.roll, axes.pitch, axes.yawrate, axes.thrust
                    ).await {
                        update_status(&ui, &format!("Commander error: {e}"));
                        break;
                    }
                }
            }
        }
    }

    // Stop motors before disconnecting
    let _ = cf.commander.setpoint_rpyt(0.0, 0.0, 0.0, 0).await;
    update_status(&ui, "Disconnecting...");
    cf.disconnect().await;
    set_connected(&ui, false);
    update_status(&ui, "Disconnected");
}

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let _guard = rt.enter();
    let link_context = Arc::new(LinkContext::new());

    let ui = MainWindow::new().unwrap();
    let cancel_token: Arc<Mutex<CancellationToken>> =
        Arc::new(Mutex::new(CancellationToken::new()));

    // Shared gamepad state
    let selected_gamepad_index = Arc::new(AtomicUsize::new(0));
    let gamepad_enabled = Arc::new(AtomicBool::new(false));
    let gamepad_axes = spawn_gamepad_thread(ui.as_weak(), selected_gamepad_index.clone());

    ui.on_connect_clicked({
        let ui_handle = ui.as_weak();
        let link_context = link_context.clone();
        let cancel_token = cancel_token.clone();
        let rt_handle = rt.handle().clone();
        let gamepad_axes = gamepad_axes.clone();
        let gamepad_enabled = gamepad_enabled.clone();
        move || {
            let ui = ui_handle.unwrap();
            if ui.get_connected() {
                // Disconnect: cancel the running task
                let token = cancel_token.lock().unwrap();
                token.cancel();
            } else {
                // Connect: create a fresh cancellation token and spawn
                let new_token = CancellationToken::new();
                *cancel_token.lock().unwrap() = new_token.clone();
                let uri = ui.get_uri().to_string();
                let ui_weak = ui_handle.clone();
                let ctx = link_context.clone();
                rt_handle.spawn(run_connection(
                    ui_weak,
                    ctx,
                    uri,
                    new_token,
                    gamepad_axes.clone(),
                    gamepad_enabled.clone(),
                ));
            }
        }
    });

    ui.on_gamepad_selected({
        let selected = selected_gamepad_index.clone();
        move |_value| {
            // The ComboBox current-index is bound in the UI;
            // we sync via the shared atomic for the gamepad thread
            // Unfortunately we get the value string, not index.
            // We'll update from the UI's current-index instead via a workaround:
            // Since the selected callback fires after the index updates,
            // we don't need to do anything here - we use a timer approach below.
            let _ = selected;
        }
    });

    // Poll gamepad index and update setpoint display periodically
    let selected_gamepad_for_timer = selected_gamepad_index.clone();
    let gamepad_axes_for_timer = gamepad_axes.clone();
    let gamepad_enabled_for_timer = gamepad_enabled.clone();
    let ui_weak_for_timer = ui.as_weak();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(50),
        move || {
            if let Some(ui) = ui_weak_for_timer.upgrade() {
                // Sync ComboBox index to the atomic
                let idx = ui.get_gamepad_index() as usize;
                selected_gamepad_for_timer.store(idx, Ordering::Relaxed);

                // Show gamepad values in setpoint column when enabled
                if gamepad_enabled_for_timer.load(Ordering::Relaxed) {
                    let axes = gamepad_axes_for_timer.lock().unwrap().clone();
                    ui.set_roll_setpoint(axes.roll);
                    ui.set_pitch_setpoint(axes.pitch);
                    ui.set_yaw_setpoint(axes.yawrate);
                }
            }
        },
    );

    ui.on_gamepad_enable_changed({
        let ui_handle = ui.as_weak();
        let enabled = gamepad_enabled.clone();
        move |checked| {
            let ui = ui_handle.unwrap();
            ui.set_gamepad_enabled(checked);
            enabled.store(checked, Ordering::Relaxed);
        }
    });

    ui.run().unwrap();
}
