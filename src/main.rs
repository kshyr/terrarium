use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::PrimaryWindow,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const NUM_ANTS: u32 = 20;
const ANT_SPEED: f32 = 100.0;
const ANT_SIZE: f32 = 4.0;
const SEARCH_FACTOR: f32 = 0.2;
const TRAIL_INTERVAL: f32 = 0.1;
const TRAIL_LIFESPAN: f32 = 15.0;
const FOOD_DISTANCE: f32 = 150.0;
const HOME: Vec2 = Vec2::new(0.0, 0.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
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
    found_food: bool,
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
                transform: Transform::from_translation(Vec3::new(HOME.x, HOME.y, 10.)),
                ..Default::default()
            },
            Ant {
                direction,
                search_timer: Timer::from_seconds(0.01, TimerMode::Repeating),
                trail_timer: Timer::from_seconds(TRAIL_INTERVAL, TimerMode::Repeating),
                found_food: false,
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

    for _ in 0..FOOD_SIZE {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(2.0).into()).into(),
                material: materials.add(ColorMaterial::from(Color::OLIVE)),
                transform: Transform::from_translation(Vec3::new(
                    random_range(-180.0, -190.0),
                    random_range(210.0, 220.0),
                    0.,
                )),
                ..Default::default()
            },
            Food {},
        ));
    }

    for _ in 0..FOOD_SIZE {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(2.0).into()).into(),
                material: materials.add(ColorMaterial::from(Color::OLIVE)),
                transform: Transform::from_translation(Vec3::new(
                    random_range(150.0, 160.0),
                    random_range(180.0, 190.0),
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
    mut food_query: Query<(&mut Transform, Entity, &mut Food), Without<Ant>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let window = window_query.get_single().unwrap();

    for (mut transform, mut ant) in ants_query.iter_mut() {
        let translation = &mut transform.translation;
        for mut food in food_query.iter_mut() {
            let mut food_translation = &mut food.0.translation;
            if (food_translation.x - translation.x).abs() < 1.0
                && (food_translation.y - translation.y).abs() < 1.0
            {
                ant.found_food = true;
                commands.entity(food.1).despawn();
            }
        }

        ant.trail_timer.tick(time.delta());
        if ant.trail_timer.finished() {
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(1.0).into()).into(),
                    material: materials.add(ColorMaterial::from(if ant.found_food {
                        Color::SEA_GREEN
                    } else {
                        Color::CRIMSON
                    })),
                    transform: Transform::from_translation(Vec3::new(
                        translation.x,
                        translation.y,
                        if ant.found_food { 5.0 } else { 1.0 },
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
        color_mat.color.set_a(trail.intensity);

        if trail.intensity <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_ant_direction(
    mut commands: Commands,
    mut ants_query: Query<(&mut Transform, &mut Ant), Without<Food>>,
    mut food_query: Query<(&mut Transform, &mut Food), Without<Ant>>,
    time: Res<Time>,
) {
    for (mut transform, mut ant) in ants_query.iter_mut() {
        let ant_pos = transform.translation;
        // let direction = Vec2::new(random_range(-1.0, 1.0), random_range(-1.0, 1.0)).normalize();

        if ant.found_food {
            if (HOME.x - ant_pos.x).abs() < 4.0 && (HOME.y - ant_pos.y).abs() < 4.0 {
                ant.found_food = false;
                ant.direction *= -1.0;
            } else {
                ant.direction = Vec2::new(HOME.x - ant_pos.x, HOME.y - ant_pos.y).normalize();
            }
        }

        ant.search_timer.tick(time.delta());
        if ant.search_timer.finished() {
            for mut food in food_query.iter_mut() {
                let food_pos = food.0.translation;

                if !ant.found_food
                    && (food_pos.x - ant_pos.x).abs() < FOOD_DISTANCE
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
