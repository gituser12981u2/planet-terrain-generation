use bevy::{
    asset::Assets,
    color::Color,
    core_pipeline::core_3d::Camera3d,
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    math::{Vec3, primitives::Cuboid},
    pbr::{
        AmbientLight, CascadeShadowConfig, CascadeShadowConfigBuilder, MeshMaterial3d,
        StandardMaterial,
    },
    render::mesh::{Mesh, Mesh3d},
    time::Time,
    transform::components::Transform,
    utils::default,
};

use crate::root::flycam::FlyCam;

#[derive(Component)]
pub struct Spinner;

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.5, 0.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Spinner,
    ));

    // light
    let cascades: CascadeShadowConfig = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 10.0, // tight near detail
        maximum_distance: 200.0,       // shadow distance from camera
        num_cascades: 4,               // 3–4
        ..default()
    }
    .into();

    commands.spawn((
        bevy::pbr::DirectionalLight {
            illuminance: 60_000.0, // reduce if still harsh; 40k–80k is sun-like
            shadows_enabled: true,
            // Biases: raise until acne disappears, then stop
            shadow_depth_bias: 0.002, // 0.001–0.01
            shadow_normal_bias: 0.8,  // 0.4–1.5
            ..default()
        },
        cascades,
        // rake the light so terrain self-shadows gently
        Transform::from_xyz(30.0, 50.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::srgb(1.0, 1.0, 1.0),
        brightness: 0.12,
        ..default()
    });

    // camera
    let tf = Transform::from_xyz(4.0, 3.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y);
    let (yaw, pitch) = yaw_pitch_from_forward(-*tf.forward());
    commands.spawn((
        Camera3d::default(),
        tf,
        FlyCam {
            yaw,
            pitch,
            captured: false,
        },
    ));
}

pub fn spin(time: Res<Time>, mut q: Query<&mut Transform, With<Spinner>>) {
    if let Ok(mut transform) = q.single_mut() {
        transform.rotate_y(1.0 * time.delta_secs());
    }
}

/// Derive yaw and pitch from a forward vector in radians.
fn yaw_pitch_from_forward(forward: Vec3) -> (f32, f32) {
    let f = forward.normalize_or_zero();
    let yaw = f.z.atan2(f.x) + std::f32::consts::FRAC_PI_2; // align to bevy's axes
    let pitch = f.y.asin();
    (yaw, pitch)
}
