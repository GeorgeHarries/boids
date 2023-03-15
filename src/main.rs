use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy::render::mesh::{self, PrimitiveTopology};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;

// Window constants
pub const ASPECT_RATIO: f32 = 16.0/9.0;
pub const WINDOW_HEIGHT: f32 =  720.0;
pub const WINDOW_WIDTH: f32 = WINDOW_HEIGHT*ASPECT_RATIO;

pub const SKY_COLOUR: Color = Color::rgb(0.2, 0.2, 0.2);
pub const BOID_COLOUR: Color = Color::WHITE;

pub const BOID_SPIN_SPEED: f32 = 0.2*std::f32::consts::PI;
pub const BOID_SLIDE_SPEED: f32 = 1.0;

////////////////////////////////////////////////////////////////
// Components
////////////////////////////////////////////////////////////////
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct Boid;

////////////////////////////////////////////////////////////////
// App
////////////////////////////////////////////////////////////////
fn main() {
    App::new()
    .insert_resource(ClearColor(SKY_COLOUR))
    .add_plugins(DefaultPlugins.set(
        WindowPlugin { 
            primary_window: Some(Window {
                title: "Bevy Boids Implementation".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }
    ))
    .add_plugin(FlyCameraPlugin)
    // .add_plugin(WorldInspectorPlugin)
    .register_type::<Boid>()
    .add_startup_system(spawn_camera)
    .add_startup_system(set_light_level)
    .add_startup_system(spawn_boid)
    .add_system(spin_boid)
    .add_system(slide_boid)
    .run();
}

////////////////////////////////////////////////////////////////
// Systems
////////////////////////////////////////////////////////////////
fn spawn_camera(
    mut commands: Commands,
) {
    commands
    .spawn(Camera3dBundle {transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y), ..default()})
    .insert(FlyCamera {sensitivity: 4.5, key_down: KeyCode::LControl, ..default()});
}

fn set_light_level(
    mut commands: Commands,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.75,
    });
}

fn spawn_boid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut boid_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    boid_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [ 0.0, -0.05,  0.0 ],
            [-0.05, 0.0,  -0.05],
            [ 0.05, 0.0,  -0.05],
            [ 0.05, 0.0,   0.05],
            [-0.05, 0.0,   0.05],
            [ 0.0,  0.5,   0.0 ]
            ],
        );

    boid_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL, 
        vec![
            [ 0.0, -1.0,  0.0],
            [-1.0, -2.5, -1.0],
            [ 1.0, -2.5, -1.0],
            [ 1.0, -2.5,  1.0],
            [-1.0, -2.5,  1.0],
            [ 0.0,  1.0,  0.0]
        ]
    );

    boid_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 0.0]; 6]
    );
    
    boid_mesh.set_indices(Some(mesh::Indices::U32(
        vec![
            0,1,2,  // Short Face 1
            0,2,3,  // Short Face 2
            0,3,4,  // Short Face 3
            0,4,1,  // Short Face 4
            1,5,2,  // Long Face 1
            2,5,3,  // Long Face 2
            3,5,4,  // Long Face 3
            4,5,1   // Long Face 4
        ]
    )));
    
    commands.spawn(PbrBundle {
            mesh: meshes.add(boid_mesh),
            material: materials.add(BOID_COLOUR.into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        }
    )
    .insert(Boid);
}

fn spin_boid(
    time: Res<Time>,
    mut boids: Query<(&Boid, &mut Transform)>,
) {
    for (_, mut transform) in boids.iter_mut() {
        transform.rotate_local_y(BOID_SPIN_SPEED*time.delta_seconds());
    }
}

fn slide_boid(
    time: Res<Time>,
    mut boids: Query<(&Boid, &mut Transform)>,
) {
    for (_, mut transform) in boids.iter_mut() {
        transform.translation.y += BOID_SLIDE_SPEED*time.delta_seconds();
    }
}
////////////////////////////////////////////////////////////////
// Helper functions
////////////////////////////////////////////////////////////////
