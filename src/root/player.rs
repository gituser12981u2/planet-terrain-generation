// use bevy::{
//     asset::Assets,
//     color::Color,
//     core_pipeline::core_3d::Camera3d,
//     ecs::{
//         component::Component,
//         system::{Commands, Query, Res, ResMut},
//     },
//     input::{ButtonInput, keyboard::KeyCode},
//     math::{EulerRot, Quat, Vec3, primitives::Capsule3d},
//     pbr::{MeshMaterial3d, StandardMaterial},
//     render::{
//         self,
//         mesh::{Capsule3dMeshBuilder, CapsuleUvProfile, Mesh, Mesh3d},
//     },
//     time::Time,
//     transform::components::Transform,
//     utils::default,
// };
// use bevy_rapier3d::prelude::{
//     CharacterLength, Collider, KinematicCharacterController, KinematicCharacterControllerOutput,
// };

// #[derive(Component)]
// pub struct Player {
//     vy: f32,
//     grounded: bool,
// }

// #[derive(Component, Default)]
// pub struct PlayerCam {
//     pub yaw: f32,
//     pub pitch: f32,
//     pub captured: bool,
// }

// pub fn spawn_player(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut mats: ResMut<Assets<StandardMaterial>>,
// ) {
//     let radius = 0.35;
//     let half_length = 0.9;
//     let eye = 0.9;

//     let player = commands
//         .spawn((
//             // Visuals
//             Mesh3d(meshes.add(Mesh::from(Capsule3dMeshBuilder {
//                 capsule: Capsule3d {
//                     radius,
//                     half_length,
//                 },
//                 uv_profile: CapsuleUvProfile::Aspect,
//                 ..default()
//             }))),
//             MeshMaterial3d(mats.add(StandardMaterial {
//                 base_color: Color::srgb(0.7, 0.7, 0.85),
//                 ..default()
//             })),
//             Transform::from_xyz(0.0, 2.0, 0.0),
//             // Physics
//             Collider::capsule_y(half_length, radius),
//             KinematicCharacterController {
//                 snap_to_ground: Some(CharacterLength::Absolute(0.2)),
//                 max_slope_climb_angle: 45.0_f32.to_radians(),
//                 min_slope_slide_angle: 30.0_f32.to_radians(),
//                 ..default()
//             },
//             Player {
//                 vy: 0.0,
//                 grounded: false,
//             },
//         ))
//         .id();

//     // Child camera entity for first-person view
//     let cam_tf = Transform::from_xyz(0.0, eye, 0.0).looking_at(Vec3::new(0.0, eye, -1.0), Vec3::Y);

//     commands
//         .spawn((
//             Camera3d::default(),
//             cam_tf,
//             PlayerCam::default(),
//             render::camera::Camera {
//                 is_active: true,
//                 ..default()
//             },
//         ))
//         .set_parent(player);
// }

// pub fn drive_character(
//     time: Res<Time>,
//     keys: Res<ButtonInput<KeyCode>>,
//     mut query: Query<(
//         &mut Transform,
//         &mut KinematicCharacterController,
//         &mut Player,
//         Option<&KinematicCharacterControllerOutput>,
//     )>,
// ) {
//     let Ok((mut tf, mut ctrl, mut player, out)) = query.single_mut() else {
//         return;
//     };

//     // read last step's ground state
//     if let Some(out) = out {
//         player.grounded = out.grounded;
//         if player.grounded && player.vy < 0.0 {
//             player.vy = 0.0;
//         }
//     }

//     // input: XZ plane in local yaw space
//     let mut wish = Vec3::ZERO;
//     if keys.pressed(KeyCode::KeyW) {
//         wish.z -= 1.0;
//     }
//     if keys.pressed(KeyCode::KeyS) {
//         wish.z += 1.0;
//     }
//     if keys.pressed(KeyCode::KeyA) {
//         wish.x -= 1.0;
//     }
//     if keys.pressed(KeyCode::KeyD) {
//         wish.x += 1.0;
//     }
//     if wish != Vec3::ZERO {
//         wish = (tf.rotation * wish.normalize())
//             .with_y(0.0)
//             .normalize_or_zero();
//     }

//     // jump
//     const GRAVITY: f32 = 30.0;
//     const JUMP_VELOCITY: f32 = 10.0;
//     if player.grounded && keys.just_pressed(KeyCode::Space) {
//         player.vy = JUMP_VELOCITY;
//         player.grounded = false;
//     } else {
//         player.vy -= GRAVITY * time.delta_secs();
//     }

//     // horizontal speed
//     let speed = if keys.pressed(KeyCode::ShiftLeft) {
//         8.0
//     } else {
//         4.5
//     };
//     let horizontal = wish * speed;

//     ctrl.translation = Some((horizontal + Vec3::Y * player.vy) * time.delta_secs());

//     if horizontal.length_squared() > 1e-6 {
//         let dir = horizontal.normalize();
//         let yaw = dir.z.atan2(dir.x) + std::f32::consts::FRAC_PI_2;
//         tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);
//     }
// }
