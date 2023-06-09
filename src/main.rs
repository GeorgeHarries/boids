use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy::render::mesh::{self, PrimitiveTopology};

use rand::distributions::{Distribution, Uniform};

// Window constants
pub const ASPECT_RATIO: f32           = 16.0/9.0;
pub const WINDOW_HEIGHT: f32          = 720.0;
pub const WINDOW_WIDTH: f32           = WINDOW_HEIGHT*ASPECT_RATIO;

// World constants
pub const SKY_COLOUR: Color           = Color::rgb(0.2, 0.2, 0.2);

// Boid constants
pub const BOID_NUM: i32               = 300;
pub const BOID_COLOUR: Color          = Color::WHITE;
pub const BOID_SPEED_SOFT_CAP: f32    = 6.0;
pub const BOID_MAX_ACCELERATION: f32  = 10.0;
pub const BOID_VISION_RANGE: f32      = 5.0;
pub const BOID_PERSONAL_SPACE: f32    = 0.5;

pub const SEPARATION_WEIGHTING: f32   = 10.0;
pub const ALLIGNMENT_WEIGHTING: f32   = 20.0;
pub const COHESION_WEIGHTING: f32     = 2.0;
pub const BOUND_WEIGHTING: f32        = 15.0;
pub const DRAG_WEIGHTING: f32         = 10.0;

pub const BOUNDS_WIDTH_X: f32         = 40.0;
pub const BOUNDS_WIDTH_Y: f32         = 40.0;
pub const BOUNDS_WIDTH_Z: f32         = 40.0;

////////////////////////////////////////////////////////////////
// Components
////////////////////////////////////////////////////////////////
#[derive(Component)]
struct Boid;

#[derive(Component)]
struct Velocity {
    vector: Vec3,
}

#[derive(Component)]
struct Acceleration {
    vector: Vec3,
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
    .add_startup_system(spawn_camera)
    .add_startup_system(set_light_level)
    .add_startup_system(spawn_boid)
    .add_systems((
        boids_calculate_acceleration,
        boids_accelerate,
        boids_move
    ).chain())
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
        brightness: 1.0,
    });
}

fn spawn_boid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    
    let boid_mesh: Mesh = get_boid_mesh();
    
    for _ in 0..BOID_NUM {
        
        let mut rng = rand::thread_rng();
        
        let x_pos = Uniform::from(-0.5*BOUNDS_WIDTH_X..0.5*BOUNDS_WIDTH_X).sample(&mut rng);
        let y_pos = Uniform::from(-0.5*BOUNDS_WIDTH_Y..0.5*BOUNDS_WIDTH_Y).sample(&mut rng);
        let z_pos = Uniform::from(-0.5*BOUNDS_WIDTH_Z..0.5*BOUNDS_WIDTH_Z).sample(&mut rng);
        
        let x_dir = Uniform::from(-0.5*BOUNDS_WIDTH_X..0.5*BOUNDS_WIDTH_X).sample(&mut rng);
        let y_dir = Uniform::from(-0.5*BOUNDS_WIDTH_Y..0.5*BOUNDS_WIDTH_Y).sample(&mut rng);
        let z_dir = Uniform::from(-0.5*BOUNDS_WIDTH_Z..0.5*BOUNDS_WIDTH_Z).sample(&mut rng);
        let mut direction: Vec3 = Vec3 {x: x_dir, y: y_dir, z: z_dir};
        direction = direction.normalize();

        commands.spawn(PbrBundle {
                mesh: meshes.add(boid_mesh.clone()),
                material: materials.add(BOID_COLOUR.into()),
                transform: Transform::from_xyz(x_pos, y_pos, z_pos),
                ..default()
            }
        )
        .insert(Boid)
        .insert(Velocity {vector: direction})
        .insert(Acceleration {vector: Vec3::ZERO});
    }
}

fn boids_calculate_acceleration(
    mut boids: Query<(Entity, &mut Acceleration, &Velocity, &Transform), With<Boid>>,
    other_boids: Query<(Entity, &Velocity, &Transform), With<Boid>>,
) {
    for (this_entity, mut acceleration, velocity, transform) in boids.iter_mut() {
        
        // Tracking vectors
        let mut avoid_positions: Vec<Vec3> = Vec::new();
        let mut local_headings: Vec<Vec3> = Vec::new();
        let mut local_positions: Vec<Vec3> = Vec::new();

        // Iterate over other boids
        for (other_entity, other_velocity, other_transform) in other_boids.iter() {
            if other_entity == this_entity {continue}
            let distance: f32 = transform.translation.distance(other_transform.translation);
            if distance < BOID_PERSONAL_SPACE {
                avoid_positions.push(other_transform.translation);
            }
            if distance > BOID_PERSONAL_SPACE && distance < BOID_VISION_RANGE {
                local_headings.push(other_velocity.vector.normalize_or_zero());
                local_positions.push(other_transform.translation);
            }
        }

        // Calculate separation direction
        let avoids: usize = avoid_positions.len();
        let mut avoid_average_pos: Vec3 = transform.translation;
        for pos in avoid_positions.into_iter() {
            avoid_average_pos += pos;
        }
        avoid_average_pos = avoid_average_pos / (avoids as f32);
        let separation: Vec3 = (avoids as f32) * (transform.translation - avoid_average_pos).normalize_or_zero();

        // Calculate allignment direction
        let locals: usize = local_headings.len();
        let mut allignment: Vec3 = Vec3::ZERO;
        for dir in local_headings.into_iter() {
            allignment += dir;
        }
        let allignment: Vec3 = (allignment / (locals as f32)).normalize_or_zero();

        // Calculate cohesion direction
        let locals: usize = local_positions.len();
        let mut local_average_pos: Vec3 = transform.translation;
        for pos in local_positions.into_iter() {
            local_average_pos += pos;
        }
        local_average_pos = local_average_pos / (locals as f32);
        let cohesion: Vec3 = (local_average_pos - transform.translation).normalize_or_zero();

        // Calculate drag direction
        let mut drag: Vec3 = Vec3::ZERO;
        if velocity.vector.length() > BOID_SPEED_SOFT_CAP {
            drag = (BOID_SPEED_SOFT_CAP-velocity.vector.length())*velocity.vector.normalize_or_zero();
        }
        let drag: Vec3 = drag;

        // Calculate bounding force direction
        let mut bound: Vec3 = Vec3::ZERO;
        if transform.translation.x >=  0.5*BOUNDS_WIDTH_X {bound.x -= 1.0}
        if transform.translation.x <= -0.5*BOUNDS_WIDTH_X {bound.x += 1.0}
        if transform.translation.y >=  0.5*BOUNDS_WIDTH_Y {bound.y -= 1.0}
        if transform.translation.y <= -0.5*BOUNDS_WIDTH_Y {bound.y += 1.0}
        if transform.translation.z >=  0.5*BOUNDS_WIDTH_Z {bound.z -= 1.0}
        if transform.translation.z <= -0.5*BOUNDS_WIDTH_Z {bound.z += 1.0}
        let bound: Vec3 = bound;

        // Combine directions
        acceleration.vector = SEPARATION_WEIGHTING*separation 
                               + ALLIGNMENT_WEIGHTING*allignment
                               + COHESION_WEIGHTING*cohesion
                               + BOUND_WEIGHTING*bound
                               + DRAG_WEIGHTING*drag;
        acceleration.vector = acceleration.vector.clamp_length_max(BOID_MAX_ACCELERATION);
    }
}

fn boids_accelerate(
    time: Res<Time>,
    mut boids: Query<(&mut Velocity, &Acceleration, &mut Transform), With<Boid>>,
) {
    for (mut velocity, acceleration, mut transform) in boids.iter_mut() {
        velocity.vector += acceleration.vector*time.delta_seconds();
        transform.look_to(velocity.vector, Vec3::Y);  // REVISIT: Think about how up is defined
    }
}

fn boids_move(
    time: Res<Time>,
    mut boids: Query<(&Velocity, &mut Transform), With<Boid>>,
) {
    for (velocity, mut transform) in boids.iter_mut() {
        transform.translation += velocity.vector*time.delta_seconds();
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