use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::gamepad::GamepadButton;
use bevy::prelude::*;
use bevy::sprite::SpriteBundle;
use rand::Rng;
use rand::rngs::ThreadRng;
use std::time::Duration;

pub const PERSONCOUNT: i32 = 5000;
pub const PERSONSPEED: f32 = 50.;
pub const PERSONSIZE: f32 = 10.;
pub const BOXSIZE: f32 = 720.;
pub const PLAYERSPEED: f32 = 100.;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
        .add_systems(Startup, (setup, spawn_player, populate))
        .add_systems(
            Update,
            (
                move_population,
                update_population_direction,
                infect,
                define_space,
                gamepad_input,
            ),
        )
        .run()
}

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(TimerRes {
        timer: Timer::new(Duration::from_secs(2), TimerMode::Repeating),
    });
}

#[derive(Resource)]
struct TimerRes {
    timer: Timer,
}

#[derive(Component)]
struct InfectTimer {
    timer: Timer,
}

#[derive(Component)]
pub struct Person {
    pub direction: Vec3,
}

#[derive(Component)]
pub struct Player {
    pub is_infected: bool,
    pub direction: Vec3,
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player {
            is_infected: false,
            direction: Vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: (Some(Vec2 {
                    x: PERSONSIZE,
                    y: PERSONSIZE,
                })),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        InfectTimer {
            timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
        },
    ));
}

fn populate(mut commands: Commands) {

    let mut rng = rand::thread_rng();

    //patient 0
    commands.spawn((
        Person {
            direction: generate_velocity(&mut rng),
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: (Some(Vec2 {
                    x: PERSONSIZE,
                    y: PERSONSIZE,
                })),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        InfectTimer {
            timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
        },
        Infected,
    ));

    let mut v = vec![];
    for _ in 0..PERSONCOUNT {
        let posx = rng.gen_range(-BOXSIZE..=BOXSIZE);
        let posy = rng.gen_range(-BOXSIZE..=BOXSIZE);

        v.push((
            Person {
                direction: generate_velocity(&mut rng),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN,
                    custom_size: (Some(Vec2 {
                        x: PERSONSIZE,
                        y: PERSONSIZE,
                    })),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(posx as f32, posy as f32, 0.)),
                ..default()
            },
            InfectTimer {
                timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
            },
        ));
    }
    commands.spawn_batch(v);
}

fn move_population(mut query: Query<(&mut Transform, &Person)>, time: Res<Time>) {
    for (mut transform, person) in &mut query.iter_mut() {
        transform.translation += person.direction * PERSONSPEED * time.delta_seconds();
    }
}

fn update_population_direction(
    mut query: Query<&mut Person>,
    time: Res<Time>,
    mut timer_res: ResMut<TimerRes>,
) {
    timer_res.timer.tick(time.delta());

    let mut rng = rand::thread_rng();
    for mut person in &mut query {
        if timer_res.timer.just_finished() {
            person.direction = generate_velocity(&mut rng);
        }
    }
}

#[derive(Component)]
struct Infected;

#[allow(clippy::type_complexity)]
fn infect(
    mut commands: Commands,
    // mut query: Query<(&mut Transform, &mut Person, &mut Sprite, &mut InfectTimer)>,
    query_infected: Query<&Transform, With<Infected>>,
    mut query_healthy: Query<(Entity, &Transform, &mut Sprite, &mut InfectTimer), (With<Person>, Without<Infected>)>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    for infected_transform in &query_infected {
        for (entity, healthy_transform, mut sprite,mut infect_timer) in &mut query_healthy {
            let distance = infected_transform.translation.distance(healthy_transform.translation);
            if distance < PERSONSIZE {
                //attempt to infect once every 1/5 second
                infect_timer.timer.tick(time.delta());
                if infect_timer.timer.finished() {
                    // 1/5 chance to infect
                    let infect = rng.gen_range(0..=4);
                    if infect == 4 {
                        sprite.color = Color::RED;
                        commands.entity(entity).insert(Infected);
                    }
                }
            }
        }
    }

    // let combinations = &mut query.iter_combinations_mut();
    // while let Some(
    //     [(tranform1, mut person1, mut sprite1, mut infect_timer1), (transform2, mut person2, mut sprite2, mut infect_timer2)],
    // ) = combinations.fetch_next()
    // {
    //     let distance = tranform1.translation.distance(transform2.translation);

    //     if (person2.is_infected || person1.is_infected) && distance < PERSONSIZE {
    //         // attempt to infect once every 1/5 second
    //         infect_timer2.timer.tick(time.delta());
    //         if infect_timer2.timer.finished() {
    //             // 1/5 chance to infect
    //             let infect = rng.gen_range(0..=4);
    //             if infect == 4 {
    //                 person1.is_infected = true;
    //                 person2.is_infected = true;
    //                 sprite1.color = Color::RED;
    //                 sprite2.color = Color::RED;
    //             }
    //         }
    //     }
    // }
}

fn define_space(mut query: Query<&mut Transform, With<Person>>) {
    let minxy = (-BOXSIZE / 2.) - PERSONSIZE / 2.;
    let maxxy = (BOXSIZE / 2.) - PERSONSIZE / 2.;

    for mut transform in query.iter_mut() {
        let mut translation = transform.translation;

        if translation.x < minxy {
            translation.x = minxy;
        } else if translation.x > maxxy {
            translation.x = maxxy
        }
        if translation.y < minxy {
            translation.y = minxy;
        } else if translation.y > maxxy {
            translation.y = maxxy
        }

        transform.translation = translation
    }
}

fn generate_velocity(rng: &mut ThreadRng) -> Vec3 {
    let velx = rng.gen_range(-1.0..1.0);
    let vely = rng.gen_range(-1.0..1.0);

    Vec3::new(velx, vely, 0.)
}

fn gamepad_input(
    buttons: Res<Input<GamepadButton>>,
    mut query: Query<&mut Transform, With<Player>>,
    gamepads: Res<Gamepads>,
    time: Res<Time>,
) {
    let Some(gamepad) = gamepads.iter().next() else { 
        return; 
    }; 

    // leafwing-input-manager

    // In a real game, the buttons would be configurable, but here we hardcode them
    let up_dpad = GamepadButton {
        gamepad,
        button_type: GamepadButtonType::DPadUp,
    };
    let down_dpad = GamepadButton {
        gamepad,
        button_type: GamepadButtonType::DPadDown,
    };
    let left_dpad = GamepadButton {
        gamepad,
        button_type: GamepadButtonType::DPadLeft,
    };
    let right_dpad = GamepadButton {
        gamepad,
        button_type: GamepadButtonType::DPadRight,
    };

    if buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South)) {
        info!("{:?} just pressed South", gamepad);
    }

    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if buttons.pressed(up_dpad) {
            direction += Vec3::new(0., 1., 0.)
        }

        if buttons.pressed(down_dpad) {
            direction += Vec3::new(0., -1., 0.)
        }

        if buttons.pressed(left_dpad) {
            direction += Vec3::new(-1., 0., 0.)
        }

        if buttons.pressed(right_dpad) {
            direction += Vec3::new(1., 0., 0.)
        }

        transform.translation += direction * PLAYERSPEED * time.delta_seconds();
    }
}
