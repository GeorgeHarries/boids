use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy::render::mesh::{self, PrimitiveTopology};

use rand::distributions::{Distribution, Uniform};

// Window constants
pub const ASPECT_RATIO: f32 = 16.0/9.0;
pub const WINDOW_HEIGHT: f32 =  720.0;
pub const WINDOW_WIDTH: f32 = WINDOW_HEIGHT*ASPECT_RATIO;

pub const SKY_COLOUR: Color = Color::rgb(0.2, 0.2, 0.2);

pub const BOIDS_NUM: i32 = 5000;

pub const BOID_COLOUR: Color = Color::WHITE;
pub const BOID_SPIN_SPEED: f32 = 6.0;
pub const BOID_SLIDE_SPEED: f32 = 10.0;

pub const BOUNDS_WIDTH_X: f32 = 50.0;
pub const BOUNDS_WIDTH_Y: f32 = 50.0;
pub const BOUNDS_WIDTH_Z: f32 = 50.0;

////////////////////////////////////////////////////////////////
// Components
////////////////////////////////////////////////////////////////
#[derive(Reflect, Component, Default)]
#[reflect(Component)]
struct Boid {
    direction: Vec3,
}

////////////////////////////////////////////////////////////////
// App
////////////////////////////////////////////////////////////////
fn main() {
    App::new()
    .insert_resource(ClearColor(SKY_COLOUR))
    .add_plugins(DefaultPlugins
        .set(WindowPlugin { 
            primary_window: Some(Window {
                title: "Bevy Boids Implementation".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        })
    )
    .add_plugin(FlyCameraPlugin)
    .register_type::<Boid>()
    .add_startup_system(spawn_camera)
    .add_startup_system(set_light_level)
    .add_startup_system(spawn_boid)
    .add_system(boids_turn)
    .add_system(boids_move)
    .add_system(boids_edge_reflect)
    .run();
}

////////////////////////////////////////////////////////////////
// Systems
////////////////////////////////////////////////////////////////
fn spawn_camera(
    mut commands: Commands,
) {
    commands
    .spawn(Camera3dBundle {transform: Transform::from_xyz(0.0, 2.5, 20.0).looking_at(Vec3::ZERO, Vec3::Y), ..default()})
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
    
    let boid_mesh: Mesh = get_boid_mesh();
    
    for _ in 0..BOIDS_NUM {
        
        let mut rng = rand::thread_rng();
        let x = Uniform::from(-0.5*BOUNDS_WIDTH_X..0.5*BOUNDS_WIDTH_X).sample(&mut rng);
        let y = Uniform::from(-0.5*BOUNDS_WIDTH_Y..0.5*BOUNDS_WIDTH_Y).sample(&mut rng);
        let z = Uniform::from(-0.5*BOUNDS_WIDTH_Z..0.5*BOUNDS_WIDTH_Z).sample(&mut rng);

        commands.spawn(PbrBundle {
                mesh: meshes.add(boid_mesh.clone()),
                material: materials.add(BOID_COLOUR.into()),
                transform: Transform::from_xyz(x, y, z),
                ..default()
            }
        )
        .insert(Boid {direction: Vec3 {x: 0.0, y: 1.0, z: 0.0}});
    }
}

fn boids_turn(
    time: Res<Time>,
    mut boids: Query<(&mut Boid, &mut Transform)>,
) {
    for (mut boid, mut transform) in boids.iter_mut() {

        // Placeholder
        // TODO: Define an acceleration function
        let acceleration: Vec3 = Vec3 {
            x:  boid.direction.y, 
            y: -boid.direction.x, 
            z:  0.0
        };
        
        boid.direction.x += BOID_SPIN_SPEED*acceleration.x*time.delta_seconds();
        boid.direction.y += BOID_SPIN_SPEED*acceleration.y*time.delta_seconds();
        boid.direction.z += BOID_SPIN_SPEED*acceleration.z*time.delta_seconds();

        boid.direction = boid.direction.normalize();

        transform.look_to(boid.direction, Vec3::Y);  // REVISIT: Think about how up is defined
    }
}

fn boids_move(
    time: Res<Time>,
    mut boids: Query<(&Boid, &mut Transform)>,
) {
    for (boid, mut transform) in boids.iter_mut() {
        transform.translation.x += boid.direction.x*BOID_SLIDE_SPEED*time.delta_seconds();
        transform.translation.y += boid.direction.y*BOID_SLIDE_SPEED*time.delta_seconds();
        transform.translation.z += boid.direction.z*BOID_SLIDE_SPEED*time.delta_seconds();
    }
}

fn boids_edge_reflect(
    mut boids: Query<(&Boid, &mut Transform)>,
) {
    for (_, mut transform) in boids.iter_mut() {
        if transform.translation.x >=  0.5*BOUNDS_WIDTH_X {transform.translation.x -= BOUNDS_WIDTH_X}
        if transform.translation.x <= -0.5*BOUNDS_WIDTH_X {transform.translation.x += BOUNDS_WIDTH_X}
        if transform.translation.y >=  0.5*BOUNDS_WIDTH_Y {transform.translation.y -= BOUNDS_WIDTH_Y}
        if transform.translation.y <= -0.5*BOUNDS_WIDTH_Y {transform.translation.y += BOUNDS_WIDTH_Y}
        if transform.translation.z >=  0.5*BOUNDS_WIDTH_Z {transform.translation.z -= BOUNDS_WIDTH_Z}
        if transform.translation.z <= -0.5*BOUNDS_WIDTH_Z {transform.translation.z += BOUNDS_WIDTH_Z}
    }
}
////////////////////////////////////////////////////////////////
// Helper functions
////////////////////////////////////////////////////////////////

fn get_boid_mesh() -> Mesh {
    let mut boid_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    boid_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [ 0.0,   0.0 ,  0.05],
            [-0.05, -0.05,  0.0 ],
            [ 0.05, -0.05,  0.0 ],
            [ 0.05,  0.05,  0.0 ],
            [-0.05,  0.05,  0.0 ],
            [ 0.0,   0.0 , -0.5 ]
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

    return boid_mesh;
}