use std::time::Duration;
use bevy::{prelude::*, time::common_conditions::on_timer};

pub const CANVAS_SIZE: Vec2 = Vec2::new(480., 270.);
pub const PLAYER_SIZE: f32 = 25.;
pub const PIPE_SPEED: f32 = 50.;
const PIPE_SIZE: Vec2 = Vec2::new(32., CANVAS_SIZE.y);
const GAP_SIZE: f32 = 100.0;

#[derive(Component)]
pub struct Pipe;

#[derive(Component)]
pub struct PipeTop;

#[derive(Component)]
pub struct PipeBottom;

#[derive(Component)]
pub struct PointsGate;

#[derive(Resource, Default)]
pub struct GameInfo {
    // 0 for Main Menu
    // 1 for in Game
    // 2 for Endscreen
    pub game_state: u32,
    pub out_of_bounds: bool,
    pub hit_ground: bool,
}

pub struct PipePlugin;

impl Plugin for PipePlugin {
    fn build(&self, app: &mut App) {
        println!("Build PipePlugin");

        app.add_systems(
            FixedUpdate,
            (
                despawn_pipes,
                shift_pipes_to_the_left,
                spawn_pipes.run_if(on_timer(
                    Duration::from_millis(2000)
                )),
                count_pipes.run_if(on_timer(
                    Duration::from_millis(2000)
                ))
            )
        );
    }
}

fn spawn_pipes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    game_info: Res<GameInfo>
) {
    if game_info.game_state == 0 {
        return;
    }

    let transform = Transform::from_xyz((CANVAS_SIZE.x / 2.) + PIPE_SIZE.x, 0., 1.);
    let gap_y_position = (time.elapsed_secs() * 4.2309875).sin() * CANVAS_SIZE.y / 5. + 20.;
    let pipe_offset: f32 = PIPE_SIZE.y / 2.0 + GAP_SIZE / 2.0;

    commands.spawn((
        transform,
        Visibility::Visible,
        Pipe,
        children![
            (
                Sprite {
                    image: asset_server.load("sprites/pipe.png"),
                    custom_size: Some(PIPE_SIZE),
                    image_mode: SpriteImageMode::Sliced(
                        TextureSlicer { 
                            border: BorderRect::axes(8., 19.), 
                            center_scale_mode:
                            SliceScaleMode::Stretch,
                            ..default()
                        },
                    ),
                    ..default()
                },
                Transform::from_xyz(0.0, pipe_offset + gap_y_position, 1.0),
                PipeTop,
            ),
            (
                Visibility::Hidden,
                Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(10.0, GAP_SIZE)),
                    ..default()
                },
                Transform::from_xyz(0.0, gap_y_position, 1.0),
                PointsGate,
            ),
            (
                Sprite {
                    image: asset_server.load("sprites/pipe.png"),
                    custom_size: Some(PIPE_SIZE),
                    image_mode: SpriteImageMode::Sliced(
                        TextureSlicer { 
                            border: BorderRect::axes(8., 19.), 
                            center_scale_mode:
                                SliceScaleMode::Stretch,
                            ..default()
                        },
                    ),
                    ..default()
                },
                Transform::from_xyz(0.0, -pipe_offset + gap_y_position, 1.0),
                PipeBottom,
            )
        ]
    ));
}

pub fn shift_pipes_to_the_left(
    mut pipes: Query<&mut Transform, With<Pipe>>,
    time: Res<Time>,
    game_info: Res<GameInfo>,
) {
    if game_info.game_state == 2 {return};
    for mut pipe in &mut pipes {
        pipe.translation.x -= PIPE_SPEED * time.delta_secs();
    }
}

fn despawn_pipes(
    mut commands: Commands,
    pipes: Query<(Entity, &Transform), With<Pipe>>,
) {
    for (entity, tranform) in pipes.iter() {
        if tranform.translation.x < -(CANVAS_SIZE.x / 2.0 + PIPE_SIZE.x) {
            commands.entity(entity).despawn();
        }
    }
}

fn count_pipes(_query: Query<&Pipe>) {
    // println!("{} pipes exist", _query.iter().len())
}