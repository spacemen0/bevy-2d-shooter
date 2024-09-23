use std::f32::consts::PI;
use std::time::Instant;

use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy::time::Stopwatch;
use rand::Rng;
use world::InGameEntity;

use crate::player::Player;
use crate::state::GameState;
use crate::*;

pub struct GunPlugin;

#[derive(Component)]
pub struct Gun;

#[derive(Component)]
pub struct GunTimer(pub Stopwatch);

#[derive(Component, Clone)]
pub enum GunType {
    Default,
    Gun1,
    Gun2,
}

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct BulletStats {
    pub speed: f32,
    pub damage: f32,
    pub lifespan: f32,
}

#[derive(Component)]
pub struct GunStats {
    pub bullets_per_shot: usize,
    pub firing_interval: f32,
    pub bullet_spread: f32,
}

#[derive(Component)]
pub struct SpawnInstant(Instant);

#[derive(Component)]
struct BulletDirection(Vec3);

#[derive(Bundle)]
pub struct GunBundle {
    pub gun: Gun,
    pub gun_timer: GunTimer,
    pub gun_type: GunType,
    pub bullet_stats: BulletStats,
    pub gun_stats: GunStats,
    pub in_game_entity: InGameEntity,
    pub sprite_bundle: SpriteBundle,
}

impl Default for GunBundle {
    fn default() -> Self {
        Self {
            gun: Gun,
            gun_timer: GunTimer(Stopwatch::new()),
            gun_type: GunType::Default,
            bullet_stats: BulletStats {
                speed: BULLET_SPEED,
                damage: BULLET_DAMAGE,
                lifespan: BULLET_TIME_SECS,
            },
            gun_stats: GunStats {
                bullets_per_shot: NUM_BULLETS_PER_SHOT,
                firing_interval: BULLET_SPAWN_INTERVAL,
                bullet_spread: BULLET_SPREAD,
            },
            in_game_entity: InGameEntity,
            sprite_bundle: Default::default(),
        }
    }
}

impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_gun_transform,
                update_bullets,
                handle_gun_firing,
                despawn_old_bullets,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

fn update_gun_transform(
    cursor_pos: Res<CursorPosition>,
    player_query: Query<&Transform, With<Player>>,
    mut gun_query: Query<&mut Transform, (With<Gun>, Without<Player>)>,
) {
    if player_query.is_empty() || gun_query.is_empty() {
        return;
    }

    let player_pos = player_query.single().translation.truncate();
    let cursor_pos = match cursor_pos.0 {
        Some(pos) => pos,
        None => player_pos,
    };
    let mut gun_transform = gun_query.single_mut();

    let angle = (player_pos.y - cursor_pos.y).atan2(player_pos.x - cursor_pos.x) + PI;
    gun_transform.rotation = Quat::from_rotation_z(angle);

    let offset = 20.0;
    let new_gun_pos = vec2(
        player_pos.x + offset * angle.cos() - 5.0,
        player_pos.y + offset * angle.sin() - 10.0,
    );

    gun_transform.translation = vec3(new_gun_pos.x, new_gun_pos.y, gun_transform.translation.z);
    gun_transform.translation.z = 15.0;
}

fn despawn_old_bullets(
    mut commands: Commands,
    bullet_query: Query<(&SpawnInstant, Entity, &BulletStats), With<Bullet>>,
) {
    for (instant, e, bullet_stats) in bullet_query.iter() {
        if instant.0.elapsed().as_secs_f32() > bullet_stats.lifespan {
            commands.add(move |world: &mut World| {
                if let Some(entity) = world.get_entity_mut(e) {
                    entity.despawn();
                }
            });
        }
    }
}

fn handle_gun_firing(
    mut commands: Commands,
    time: Res<Time>,
    mut gun_query: Query<(&Transform, &mut GunTimer, &GunType, &BulletStats, &GunStats), With<Gun>>,
    handle: Res<GlobalTextureAtlas>,
) {
    if gun_query.is_empty() {
        return;
    }

    let (gun_transform, mut gun_timer, gun_type, bullet_stats, gun_stats) = gun_query.single_mut();

    gun_timer.0.tick(time.delta());

    if gun_timer.0.elapsed_secs() >= gun_stats.firing_interval {
        gun_timer.0.reset();
        let gun_pos = gun_transform.translation.truncate();
        let mut rng = rand::thread_rng();
        let bullet_direction = gun_transform.local_x();
        match gun_type {
            GunType::Default => {
                for _ in 0..gun_stats.bullets_per_shot {
                    let dir = vec3(
                        bullet_direction.x
                            + rng.gen_range(-gun_stats.bullet_spread..gun_stats.bullet_spread),
                        bullet_direction.y
                            + rng.gen_range(-gun_stats.bullet_spread..gun_stats.bullet_spread),
                        bullet_direction.z,
                    );
                    commands.spawn((
                        SpriteBundle {
                            texture: handle.image.clone().unwrap(),
                            transform: Transform::from_translation(vec3(gun_pos.x, gun_pos.y, 1.0))
                                .with_scale(Vec3::splat(SPRITE_SCALE_FACTOR)),
                            ..default()
                        },
                        TextureAtlas {
                            layout: handle.layout.clone().unwrap(),
                            index: 16,
                        },
                        Bullet,
                        BulletDirection(dir),
                        BulletStats {
                            speed: bullet_stats.speed,
                            damage: bullet_stats.damage,
                            lifespan: bullet_stats.lifespan,
                        },
                        gun_type.clone(),
                        SpawnInstant(Instant::now()),
                    ));
                }
            }
            GunType::Gun1 => todo!(),
            GunType::Gun2 => todo!(),
        }
    }
}

fn update_bullets(
    mut bullet_query: Query<
        (&mut Transform, &BulletDirection, &BulletStats, &GunType),
        With<Bullet>,
    >,
) {
    if bullet_query.is_empty() {
        return;
    }

    for (mut t, dir, stats, gun_type) in bullet_query.iter_mut() {
        match gun_type {
            GunType::Default => {
                t.translation += dir.0.normalize() * Vec3::splat(stats.speed);
                t.translation.z = 10.0;
            }
            GunType::Gun1 => todo!(),
            GunType::Gun2 => todo!(),
        }
    }
}
