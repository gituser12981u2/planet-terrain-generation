use bevy::{
    core_pipeline::core_3d::Camera3d,
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Query, Res},
    },
    input::{ButtonInput, keyboard::KeyCode, mouse::MouseMotion},
    math::{EulerRot, Quat, Vec2, Vec3},
    time::Time,
    transform::components::Transform,
    window::{CursorGrabMode, Window},
};

const MOVE_SPEED: f32 = 8.0;
const MOVE_SPEED_FAST: f32 = 50.0;
const MOUSE_SENSITIVITY: f32 = 0.002;

#[derive(Component, Default)]
pub struct FlyCam {
    pub yaw: f32,
    pub pitch: f32,
    pub captured: bool,
}

pub fn toggle_capture(
    keys: Res<ButtonInput<KeyCode>>,
    mut win_q: Query<&mut Window>,
    mut cam_q: Query<&mut FlyCam, With<Camera3d>>,
) {
    let Ok(mut fly) = cam_q.single_mut() else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }

    fly.captured = !fly.captured;
    if let Ok(mut w) = win_q.single_mut() {
        if fly.captured {
            w.cursor_options.grab_mode = CursorGrabMode::Locked;
            w.cursor_options.visible = false;
        } else {
            w.cursor_options.grab_mode = CursorGrabMode::None;
            w.cursor_options.visible = true;
        }
    }
}

pub fn flycam_mouse_look(
    mut ev_motion: EventReader<MouseMotion>,
    mut q: Query<(&mut FlyCam, &mut Transform), With<Camera3d>>,
) {
    let Ok((mut fly, mut tf)) = q.single_mut() else {
        return;
    };

    if !fly.captured {
        ev_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for m in ev_motion.read() {
        delta += m.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }

    fly.yaw -= delta.x * MOUSE_SENSITIVITY;
    fly.pitch = (fly.pitch - delta.y * MOUSE_SENSITIVITY).clamp(-1.55, 1.55);

    tf.rotation = Quat::from_euler(EulerRot::YXZ, fly.yaw, fly.pitch, 0.0);
}

pub fn flycam_move(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut tf) = q.single_mut() else {
        return;
    };

    let mut v = Vec3::ZERO;
    let forward: Vec3 = *tf.forward();
    let right: Vec3 = *tf.right();
    let up = Vec3::Y;

    if keys.pressed(KeyCode::KeyW) {
        v += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        v -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        v += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        v -= right;
    }
    if keys.pressed(KeyCode::KeyE) {
        v += up;
    }
    if keys.pressed(KeyCode::KeyQ) {
        v -= up;
    }

    let speed = if keys.pressed(KeyCode::ShiftLeft) {
        MOVE_SPEED_FAST
    } else {
        MOVE_SPEED
    };
    tf.translation += v.normalize_or_zero() * speed * time.delta_secs();
}
