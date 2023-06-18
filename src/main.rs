use std::process::exit;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig, prelude::*, sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};

const NUM_ANTS: u32 = 10;
const ANT_SPEED: f32 = 100.0;
const ANT_SIZE: f32 = 4.0;
const SEARCH_FACTOR: f32 = 0.2;
const TRAIL_INTERVAL: f32 = 0.01;
const TRAIL_LIFESPAN: f32 = 10.0;
const FOOD_DISTANCE: f32 = 150.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_ants)
        .add_startup_system(spawn_food)
        .add_system(update_trails)
        .add_system(move_ants)
        .add_system(update_ant_direction)
        .run();
}

#[derive(Component)]
pub struct Ant {
    direction: Vec2,
    search_timer: Timer,
    trail_timer: Timer,
}

#[derive(Component)]
pub struct HomeTrail {
    intensity: f32,
}

#[derive(Component)]
pub struct Food {}

fn spawn_ants(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in 0..NUM_ANTS {
        let direction = Vec2::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0)).normalize();
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(ANT_SIZE).into()).into(),
                material: materials.add(ColorMaterial::from(Color::BEIGE)),
                transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                ..Default::default()
            },
            Ant {
                direction,
                search_timer: Timer::from_seconds(0.01, TimerMode::Repeating),
                trail_timer: Timer::from_seconds(TRAIL_INTERVAL, TimerMode::Repeating),
            },
        ));
    }
}

fn spawn_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    const FOOD_SIZE: u32 = 32;
    for _ in 0..FOOD_SIZE {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(2.0).into()).into(),
                material: materials.add(ColorMaterial::from(Color::OLIVE)),
                transform: Transform::from_translation(Vec3::new(
                    random_range(-200.0, -210.0),
                    random_range(-200.0, -210.0),
                    0.,
                )),
                ..Default::default()
            },
            Food {},
        ));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
        },
        ..Default::default()
    });
}

fn move_ants(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut ants_query: Query<(&mut Transform, &mut Ant), Without<Food>>,
    mut food_query: Query<(&mut Transform, &mut Food), Without<Ant>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let window = window_query.get_single().unwrap();

    for (mut transform, mut ant) in ants_query.iter_mut() {
        let translation = &mut transform.translation;
        // for mut food in food_query.iter_mut() {
        //     let mut food_translation = &mut food.0.translation;
        //     if (food_translation.x - translation.x).abs() < 1.0
        //         && (food_translation.y - translation.y).abs() < 1.0
        //     {
        //         food_translation.x -= 50.0 * time.delta_seconds();
        //         food_translation.y -= 50.0 * time.delta_seconds();
        //     }
        // }
        //
        ant.trail_timer.tick(time.delta());
        if ant.trail_timer.finished() {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(1.0).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::CRIMSON)),
                    transform: Transform::from_translation(Vec3::new(
                        translation.x,
                        translation.y,
                        0.,
                    )),
                    ..Default::default()
                },
                HomeTrail { intensity: 1.0 },
            ));
        }

        translation.x += ant.direction.x * ANT_SPEED * time.delta_seconds();
        translation.y += ant.direction.y * ANT_SPEED * time.delta_seconds();
        translation.x = translation
            .x
            .clamp(-window.width() / 2.0, window.width() / 2.0);
        translation.y = translation
            .y
            .clamp(-window.height() / 2.0, window.height() / 2.0);

        if translation.x.abs() == window.width() / 2.0 {
            ant.direction.x *= random_range(-1.0, 1.0);
        }
        if translation.y.abs() == window.height() / 2.0 {
            ant.direction.y *= random_range(-1.0, 1.0);
        }
    }
}

fn update_trails(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut trail_query: Query<(
        &mut Transform,
        &mut HomeTrail,
        Entity,
        &mut Handle<ColorMaterial>,
    )>,
    time: Res<Time>,
) {
    for (mut transform, mut trail, entity, mut color) in trail_query.iter_mut() {
        trail.intensity -= 1.0 / TRAIL_LIFESPAN * time.delta_seconds();
        let mut color_mat = materials.get_mut(&color).unwrap();
        color_mat.color = Color::rgba(0.8, 0.3, 0.3, trail.intensity);

        if trail.intensity <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_ant_direction(
    mut ants_query: Query<(&mut Transform, &mut Ant), Without<Food>>,
    mut food_query: Query<(&mut Transform, &mut Food), Without<Ant>>,
    time: Res<Time>,
) {
    for (mut transform, mut ant) in ants_query.iter_mut() {
        let ant_pos = transform.translation;
        // let direction = Vec2::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0)).normalize();

        ant.search_timer.tick(time.delta());
        if ant.search_timer.finished() {
            for mut food in food_query.iter_mut() {
                let food_pos = food.0.translation;
                if (food_pos.x - ant_pos.x).abs() < FOOD_DISTANCE
                    && (food_pos.y - ant_pos.y).abs() < FOOD_DISTANCE
                {
                    ant.direction =
                        Vec2::new(food_pos.x - ant_pos.x, food_pos.y - ant_pos.y).normalize();
                }
            }

            ant.direction = Vec2::new(
                random_range(
                    ant.direction.x - SEARCH_FACTOR,
                    ant.direction.x + SEARCH_FACTOR,
                ),
                random_range(
                    ant.direction.y - SEARCH_FACTOR,
                    ant.direction.y + SEARCH_FACTOR,
                ),
            )
            .normalize();
            ant.search_timer.reset();
        }
    }
}

fn random_range(min: f32, max: f32) -> f32 {
    rand::random::<f32>() * (max - min) + min
}
