use std::time::Duration;

use bevy::{
    image::ImageSamplerDescriptor, prelude::*, time::common_conditions::on_timer,
};
use flappy_bird::*;

const BG_SCROLL_SPEED: f32 = 20.0;
const BG_X: f32 = 288. * (CANVAS_SIZE.y / 512.);
const BG_Y: f32 = 512. * (CANVAS_SIZE.y / 512.); 
const BG_SIZE: Vec2 = Vec2::new(BG_X, BG_Y);
const GR_X: f32 = 336. * 0.5;
const GR_Y: f32 = 112. * 0.5;
const GR_SIZE: Vec2 = Vec2::new(GR_X, GR_Y);

#[derive(Component)]
#[require(Gravity(350.), Velocity, BirdFlap)]
struct Player;

#[derive(Component, Default, Debug)]
struct BirdFlap(usize);

#[derive(Resource)]
struct BirdAnimationFrames([Handle<Image>; 4]);

#[derive(Resource)]
struct CounterNums([Handle<Image>; 10]);

#[derive(Component)]
struct Gravity(f32);

#[derive(Component, Default)]
struct Velocity(f32);

#[derive(Event)]
struct EndGame;

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Event)]
pub struct ScorePoint;

#[derive(Component)]
struct ScoreDisplay;

#[derive(Component)]
struct BackgroundTile;

#[derive(Event)]
struct StartGame;

#[derive(Component)]
struct Instructions;

#[derive(Component)]
struct GroundTile;



fn main() {
    println!("Game starting...");
    App::new()
        .init_resource::<Score>()
        .init_resource::<GameInfo>()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler:
                ImageSamplerDescriptor::nearest()
        }))
        .add_plugins(PipePlugin)
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, (
            gravity,
            check_in_bounds,
            check_collisions,
            animate_bird.run_if(on_timer(Duration::from_millis(100)))
        ))
        .add_systems(Update, (
            on_click,
            update_score.run_if(resource_changed::<Score>),
            scroll_background,
            scroll_ground,
            enforce_bird_direction,
        ))
        .add_observer(endgame)
        .add_observer(score_point)
        .add_observer(start_game)
        .run();

}

fn startup(
    mut commands: Commands,
    mut game_info: ResMut<GameInfo>,
    asset_server: Res<AssetServer>,
) { 
    let bird_frames: BirdAnimationFrames = BirdAnimationFrames([
        asset_server.load("sprites/yellowbird-0.png"),
        asset_server.load("sprites/yellowbird-1.png"),
        asset_server.load("sprites/yellowbird-2.png"),
        asset_server.load("sprites/yellowbird-1.png"),
    ]);
    let nums: CounterNums = CounterNums([
        asset_server.load("sprites/0.png"),
        asset_server.load("sprites/1.png"),
        asset_server.load("sprites/2.png"),
        asset_server.load("sprites/3.png"),
        asset_server.load("sprites/4.png"),
        asset_server.load("sprites/5.png"),
        asset_server.load("sprites/6.png"),
        asset_server.load("sprites/7.png"),
        asset_server.load("sprites/8.png"),
        asset_server.load("sprites/9.png"),
    ]);
    commands.insert_resource(bird_frames);
    commands.insert_resource(nums);

    game_info.game_state = 0;

    // SPAWN THE CAMERA
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection { 
            scaling_mode: bevy::camera::ScalingMode::AutoMax {
                max_width: CANVAS_SIZE.x,
                max_height: CANVAS_SIZE.y,
            },
            ..OrthographicProjection::default_2d()
        })
    ));

    // SPAWN THE PLAYER
    commands.spawn((
        Player,
        Sprite {
            // custom_size: Some(),
            image: asset_server.load("sprites/yellowbird-0.png"),
            custom_size: Some(Vec2::new(34.0 * 0.75, 24.0 * 0.75)),
            ..default()
        },
        Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 2.0),
    ));

    // SPAWN INSTRUCTIONS
    commands.spawn((
        Instructions,
        Sprite {
            image: asset_server.load("sprites/message.png"),
            custom_size: Some(Vec2::new(184.0 * 0.75, 267.0 * 0.75)),

            ..default()
        },
        Transform::from_xyz(CANVAS_SIZE.x/4., 0.0, 2.0)
    ));

    // SPAWN THE BACKGROUND
    for i in 0..12 {
        commands.spawn((
            BackgroundTile,
            Sprite {
                image: asset_server.load("sprites/background-day.png"),
                custom_size: Some(BG_SIZE),
                ..default()
            },
            Transform::from_xyz((i as f32 * BG_X) - CANVAS_SIZE.x, 0.0, 0.0)
        ));
    }

    // SPAWN THE GROUND
    for i in 0..10 {
        commands.spawn((
            GroundTile,
            Sprite {
                image: asset_server.load("sprites/base.png"),
                custom_size: Some(GR_SIZE),
                ..default()
            },
            Transform::from_xyz((i as f32 * GR_X) - CANVAS_SIZE.x, -CANVAS_SIZE.y*0.5 + 5.0, 3.0)
        ));
    }

}

fn start_game(
    _: On<StartGame>,
    mut commands: Commands,
    instructions: Query<Entity, With<Instructions>>,
) {
    println!("Start game");
    for entity in instructions {
        commands.entity(entity).despawn();
    }
}

fn check_in_bounds(
    player: Single<&Transform, With<Player>>,
    mut game_info: ResMut<GameInfo>,

) {
    if game_info.game_state == 2 {
        return;
    }
    if player.translation.y > CANVAS_SIZE.y / 2.0 + PLAYER_SIZE {
        game_info.out_of_bounds = true;
    } else {
        game_info.out_of_bounds = false;
    }
}

fn on_click(
    mut velocity: Single<&mut Velocity, With<Player>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut game_info: ResMut<GameInfo>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
    player: Single<Entity, With<Player>>,
    pipes: Query<Entity, With<Pipe>>,
    counters: Query<Entity, With<ScoreDisplay>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if game_info.game_state == 0 {
            game_info.game_state = 1;
            score.0 = 0;
            commands.trigger(StartGame);
        } else if game_info.game_state == 2 {
            score.0 = 0;
            commands.entity(*player).insert((
                Transform::from_xyz(-CANVAS_SIZE.x / 4.0, 0.0, 1.0),
                Velocity(0.),
            ));

            for entity in &pipes {
                commands.entity(entity).despawn();
            }
            for entity in &counters {
                commands.entity(entity).despawn();
            }
            commands.trigger(StartGame);
            game_info.game_state = 0;
            game_info.hit_ground = false;
            return();
        }
        if game_info.out_of_bounds == false {
            velocity.0 = 150.;
        }

        commands.spawn((
            AudioPlayer::<AudioSource>(asset_server.load("audio/wing.ogg")),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn gravity(
    mut transforms: Query<(
        &mut Transform,
        &mut Velocity,
        &Gravity,
    )>,
    time: Res<Time>,
    game_info: Res<GameInfo>,
) {
    if game_info.game_state == 0 || game_info.hit_ground == true {
        return()
    } else {
        for (mut transform, mut velocity, gravity) in &mut transforms {
            velocity.0 -= gravity.0 * time.delta_secs();
            transform.translation.y += velocity.0 * time.delta_secs()
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    player: Single<(&Sprite, Entity), With<Player>>,
    player_entity: Single<Entity, With<Player>>,
    pipe_segments: Query<(&Sprite, Entity), Or<(With<PipeTop>, With<PipeBottom>)>>,
    pipe_gaps: Query<(&Sprite, Entity), With<PointsGate>>,
    transform_helper: TransformHelper,
    asset_server: Res<AssetServer>,
    mut game_info: ResMut<GameInfo>,
    ground_tiles: Query<(&Sprite, Entity), With<GroundTile>>,
) -> Result<()> {

    let player_transform = transform_helper.compute_global_transform(player.1)?;
    let player_collider = BoundingCircle::new(player_transform.translation().xy(), PLAYER_SIZE/2.);

    // gizmos.circle_2d(player_transform.translation().xy(), PLAYER_SIZE / 2., RED_400);

    for (sprite, entity) in &pipe_segments {
        let pipe_transform = transform_helper.compute_global_transform(entity)?;
        let pipe_collider = Aabb2d::new(
            pipe_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.
        );

        if player_collider.intersects(&pipe_collider) {
            commands.spawn((
                AudioPlayer::<AudioSource>(asset_server.load("audio/hit.ogg")),
                PlaybackSettings::DESPAWN,
            ));
            if game_info.game_state == 1 {
                commands.trigger(EndGame);
            }
        }
    };

    for (sprite, entity) in &pipe_gaps {
        let gap_transform = transform_helper
            .compute_global_transform(entity)?;
        let gap_collider = Aabb2d::new(
            gap_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.,
        );

        if player_collider.intersects(&gap_collider) {
            println!("point!");
            commands.trigger(ScorePoint);
            commands.entity(entity).despawn();
        }
    }

    for (sprite, entity) in &ground_tiles {
        let gap_transform = transform_helper
            .compute_global_transform(entity)?;
        let gap_collider = Aabb2d::new(
            gap_transform.translation().xy(),
            sprite.custom_size.unwrap() / 2.,
        );

        if player_collider.intersects(&gap_collider) {
            println!("touching ground");
            if game_info.game_state == 1 {
                commands.trigger(EndGame);
            }

            commands.entity(*player_entity).insert((
                Velocity(0.),
            ));

            game_info.hit_ground = true;

        }
    }

    Ok(())
}

fn score_point(
    _: On<ScorePoint>,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    score.0 += 1;
    commands.spawn((
        AudioPlayer::<AudioSource>(asset_server.load("audio/point.ogg")),
        PlaybackSettings::DESPAWN,
    ));
}

fn update_score(
    counters: Query<Entity, With<ScoreDisplay>>,
    score: Res<Score>,
    counter_nums: Res<CounterNums>,
    mut commands: Commands,
    game_info: Res<GameInfo>
) {
    if game_info.game_state == 0 {return};
    for counter in counters {
        commands.entity(counter).despawn();
    }
    let len: usize = score.0.to_string().len();
    println!("LEN: {}", len);
    for (i, n) in score.0.to_string().chars().enumerate() {
        commands.spawn((
            Sprite {
                image: counter_nums.0[n.to_digit(10).unwrap() as usize].clone(),
                ..default()
            },
            ScoreDisplay,
            Transform::from_xyz((i as f32 - (len as f32 - 1.0) / 2.0) * 28.0, CANVAS_SIZE.y / 2.0 * 0.75, 5.0)
        ));
    };
}

fn enforce_bird_direction(
    mut player: Single<
        (&mut Transform, &Velocity),
        With<Player>,
    >,
    game_info: Res<GameInfo>,
) {
    if game_info.hit_ground == true {
        player.0.rotation = Quat::from_rotation_z(-90.0);
        return;
    }
    let calculated_velocity =
        Vec2::new(PIPE_SPEED * 4., player.1.0);
    player.0.rotation = Quat::from_rotation_z(calculated_velocity.to_angle());
}

fn animate_bird(
    mut query: Query<(&mut Sprite, &mut BirdFlap), With<Player>>,
    frames: Res<BirdAnimationFrames>,
    game_info: Res<GameInfo>,
) {
    if game_info.hit_ground == true {return}
    for (mut sprite, mut birdflap) in &mut query {
        birdflap.0 = (birdflap.0 + 1) % 3;
        sprite.image = frames.0[birdflap.0].clone();
        sprite.custom_size = Some(Vec2::new(34.0 * 0.75, 24.0 * 0.75));
    };
}

fn scroll_background(
    mut query: Query<&mut Transform, With<BackgroundTile>>,
    time: Res<Time>,
    game_info: Res<GameInfo>
) {
    if game_info.game_state == 2 {return};
    let half_canvas_x = CANVAS_SIZE.x / 2.0;

    for mut transform in &mut query {
        transform.translation.x -= BG_SCROLL_SPEED * time.delta_secs();

        if transform.translation.x + (BG_X / 2.0) < -half_canvas_x  {
            transform.translation.x += BG_X * 12.0;
        }
    }
}

fn scroll_ground(
    mut query: Query<&mut Transform, With<GroundTile>>,
    time: Res<Time>,
    game_info: Res<GameInfo>,
) {
    if game_info.game_state == 2 {return};
    let half_canvas_x = CANVAS_SIZE.x / 2.0;

    for mut transform in &mut query {
        transform.translation.x -= PIPE_SPEED * time.delta_secs();

        if transform.translation.x + (GR_X / 2.0) < -half_canvas_x  {
            transform.translation.x += GR_X * 10.0;
        }
    }
}

fn endgame(
    _: On<EndGame>,
    mut commands: Commands,
    mut game_info: ResMut<GameInfo>,
    asset_server: Res<AssetServer>
) {
    println!("DEAD");

    game_info.game_state = 2;

    commands.spawn((
        AudioPlayer::<AudioSource>(asset_server.load("audio/die.ogg")),
        PlaybackSettings::DESPAWN
    ));
}