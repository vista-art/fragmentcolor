use fragmentcolor::App;
use winit::window::WindowId;

fn main() {
    let mut app = App::new();

    // Primary window event listener wrappers,
    // to be used in 1-window applications (most examples)
    app.on_moved(|_, _| {})
        .on_destroyed(|_| {})
        .on_dropped_file(|_, _| {})
        .on_hovered_file(|_, _| {})
        .on_hovered_file_cancelled(|_| {})
        .on_focused(|_, _| {})
        .on_keyboard_input(|_, _, _, _| {})
        .on_modifiers_changed(|_, _| {})
        .on_ime(|_, _| {})
        .on_cursor_moved(|_, _, _| {})
        .on_cursor_entered(|_, _| {})
        .on_cursor_left(|_, _| {})
        .on_mouse_wheel(|_, _, _, _| {})
        .on_mouse_input(|_, _, _, _| {})
        .on_axis_motion(|_, _, _, _| {})
        .on_touch(|_, _| {})
        .on_resize(|_, _| {})
        .on_scale_factor_changed(|_, _, _| {})
        .on_theme_changed(|_, _| {})
        .on_occluded(|_, _| {})
        .on_redraw_requested(|_| {})
        .on_close_requested(|_| {})
        .on_activation_token_done(|_, _, _| {})
        .on_pinch_gesture(|_, _, _, _| {})
        .on_pan_gesture(|_, _, _, _| {})
        .on_double_tap_gesture(|_, _| {})
        .on_rotation_gesture(|_, _, _, _| {})
.on_touchpad_pressure(|_, _, _, _| {});

    // Device-level handlers (no window association)
    app.on_device_added(|_, _| {})
        .on_device_removed(|_, _| {})
        .on_device_mouse_motion(|_, _, _| {})
        .on_device_mouse_wheel(|_, _, _| {})
        .on_device_motion(|_, _, _, _| {})
        .on_device_button(|_, _, _, _| {})
        .on_device_key(|_, _, _| {});

    // Per-window event listener wrappers, for multi-window applications.
    let id: WindowId = 1u64.into();
    app.on_window_moved(id, |_, _, _| {})
        .on_window_destroyed(id, |_, _| {})
        .on_window_dropped_file(id, |_, _, _| {})
        .on_window_hovered_file(id, |_, _, _| {})
        .on_window_hovered_file_cancelled(id, |_, _| {})
        .on_window_focused(id, |_, _, _| {})
        .on_window_keyboard_input(id, |_, _, _, _, _| {})
        .on_window_modifiers_changed(id, |_, _, _| {})
        .on_window_ime(id, |_, _, _| {})
        .on_window_cursor_moved(id, |_, _, _, _| {})
        .on_window_cursor_entered(id, |_, _, _| {})
        .on_window_cursor_left(id, |_, _, _| {})
        .on_window_mouse_wheel(id, |_, _, _, _, _| {})
        .on_window_mouse_input(id, |_, _, _, _, _| {})
        .on_window_axis_motion(id, |_, _, _, _, _| {})
        .on_window_touch(id, |_, _, _| {})
        .on_window_resize(id, |_, _, _| {})
        .on_window_scale_factor_changed(id, |_, _, _, _| {})
        .on_window_theme_changed(id, |_, _, _| {})
        .on_window_occluded(id, |_, _, _| {})
        .on_window_redraw_requested(id, |_, _| {})
        .on_window_close_requested(id, |_, _| {})
        .on_window_activation_token_done(id, |_, _, _, _| {})
        .on_window_pinch_gesture(id, |_, _, _, _, _| {})
        .on_window_pan_gesture(id, |_, _, _, _, _| {})
        .on_window_double_tap_gesture(id, |_, _, _| {})
        .on_window_rotation_gesture(id, |_, _, _, _, _| {})
        .on_window_touchpad_pressure(id, |_, _, _, _, _| {});

    // This file is for compile-time API healthcheck only.
}
