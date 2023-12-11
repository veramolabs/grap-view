use bevy::prelude::*;
use bevy_easings::*;
use bevy_panorbit_camera::PanOrbitCamera;
use rand::Rng;

use crate::{
    assets::MyAssets,
    events::{SelectRandomConnectedIdentifierEvent, SelectRandomIdentifierEvent},
    resources::Configuration,
};

#[derive(Component)]
pub struct Identifier;

#[derive(Component, Copy, Clone, Debug, Reflect)]
pub struct Connection {
    pub from: Entity,
    pub to: Entity,
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource, Default)]
pub struct SelectedIdentifier(pub Option<Entity>);

pub struct IdentifiersPlugin;

impl Plugin for IdentifiersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedIdentifier>()
            .register_type::<SelectedIdentifier>()
            .add_systems(Update, select_random_identifier)
            .add_systems(Update, select_random_connected_identifier)
            .add_systems(Update, update_identifiers_and_connections)
            .add_systems(Update, zoom_camera_to_selected_identifier);
    }
}

fn select_random_connected_identifier(
    mut selected_identifier: ResMut<SelectedIdentifier>,
    mut ev_rnd_c: EventReader<SelectRandomConnectedIdentifierEvent>,
    connection_query: Query<(Entity, &Connection), With<Connection>>,
) {
    #[allow(deprecated)]
    for _ in ev_rnd_c.iter() {
        // connections that have selected identifier as from or to
        let connections: Vec<(Entity, &Connection)> = connection_query
            .iter()
            .filter(|(_, connection)| {
                if let Some(selected_identifier) = selected_identifier.0 {
                    connection.from == selected_identifier || connection.to == selected_identifier
                } else {
                    false
                }
            })
            .collect();

        if connections.is_empty() {
            return;
        }
        // randomly select identifier from connections that is not the original selected identifier
        let mut rng = rand::thread_rng();
        if let Some(random_connection) = connections.get(rng.gen_range(0..connections.len())) {
            if let Some(currently_selected_identifier) = selected_identifier.0 {
                if random_connection.1.from == currently_selected_identifier {
                    selected_identifier.0 = Some(random_connection.1.to);
                } else {
                    selected_identifier.0 = Some(random_connection.1.from);
                }
            }
        };
    }
}

fn select_random_identifier(
    mut selected_identifier: ResMut<SelectedIdentifier>,
    mut ev_rnd: EventReader<SelectRandomIdentifierEvent>,
    query: Query<(Entity, &Identifier)>,
) {
    #[allow(deprecated)]
    for _ in ev_rnd.iter() {
        let identifier_count = query.iter().count() as u32;
        let mut rng = rand::thread_rng();
        let random_identifier = query
            .iter()
            .nth(rng.gen_range(0..identifier_count as usize));

        if let Some((entity, _)) = random_identifier {
            selected_identifier.0 = Some(entity);
            info!("Selecting identifier {:?}", entity);
        }
    }
}

fn zoom_camera_to_selected_identifier(
    mut commands: Commands,
    configuration: Res<Configuration>,
    selected_identifier: Res<SelectedIdentifier>,
    identifier_query: Query<&Transform, With<Identifier>>,
    mut camera_query: Query<(Entity, &Transform), With<PanOrbitCamera>>,
) {
    if !selected_identifier.is_changed() {
        return;
    };

    if let Some(id) = selected_identifier.0 {
        if let Ok(&identifier_transform) = identifier_query.get(id) {
            if let Ok((camera_entity, &camera_transform)) = camera_query.get_single_mut() {
                let direction = identifier_transform.translation - Vec3::ZERO;
                let normalized_direction = direction.normalize();
                let desired_distance = 3.0;

                let camera_position =
                    identifier_transform.translation + normalized_direction * desired_distance;

                let mid_point = camera_transform
                    .translation
                    .lerp(identifier_transform.translation, 0.9);

                commands.entity(camera_entity).insert(
                    camera_transform
                        .ease_to(
                            Transform::from_translation(mid_point)
                                .looking_at(identifier_transform.translation, Vec3::Y),
                            EaseFunction::QuinticInOut,
                            bevy_easings::EasingType::Once {
                                duration: (std::time::Duration::from_secs(
                                    configuration.animation_duration / 2,
                                )),
                            },
                        )
                        .ease_to(
                            Transform::from_xyz(
                                camera_position.x,
                                camera_position.y,
                                camera_position.z,
                            )
                            .looking_at(identifier_transform.translation, Vec3::Y),
                            EaseFunction::QuarticInOut,
                            bevy_easings::EasingType::Once {
                                duration: (std::time::Duration::from_secs(
                                    configuration.animation_duration / 2,
                                )),
                            },
                        ),
                );
            };
        };
    }
}

fn update_identifiers_and_connections(
    mut commands: Commands,
    my_assets: ResMut<MyAssets>,
    // configuration: Res<Configuration>,
    selected_identifier: Res<SelectedIdentifier>,
    identifier_query: Query<(Entity, &Transform), With<Identifier>>,
    connection_query: Query<(Entity, &Connection), With<Connection>>,
) {
    if !selected_identifier.is_changed() {
        return;
    };

    if let Some(id) = selected_identifier.0 {
        if let Ok((identifier, &identifier_transform)) = identifier_query.get(id) {
            commands.entity(identifier).insert(MaterialMeshBundle {
                mesh: my_assets.identifier_mesh_handle.clone(),
                material: my_assets.identifier_selected_material_handle.clone(),
                transform: identifier_transform.with_scale(Vec3::new(1.0, 1.0, 1.0)),
                ..Default::default()
            });
        }

        // scale all other identifiers
        for (identifier, &identifier_transform) in
            identifier_query.iter().filter(|(entity, _)| *entity != id)
        {
            commands.entity(identifier).insert(MaterialMeshBundle {
                mesh: my_assets.identifier_mesh_handle.clone(),
                material: my_assets.identifier_material_handle.clone(),
                transform: identifier_transform.with_scale(Vec3::new(0.5, 0.5, 0.5)),
                ..Default::default()
            });
        }

        // show only connections that have from or to as  selected identifier
        for (connection_entity, &connection) in connection_query.iter() {
            if connection.from == id || connection.to == id {
                if let Ok((entity, &transform)) = identifier_query.get(connection.to) {
                    if entity != id {
                        commands.entity(entity).insert(MaterialMeshBundle {
                            mesh: my_assets.identifier_mesh_handle.clone(),
                            material: my_assets.identifier_connected_material_handle.clone(),
                            transform: transform.with_scale(Vec3::new(1.0, 1.0, 1.0)),
                            ..Default::default()
                        });
                    }
                }
                if let Ok((entity, &transform)) = identifier_query.get(connection.from) {
                    if entity != id {
                        commands.entity(entity).insert(MaterialMeshBundle {
                            mesh: my_assets.identifier_mesh_handle.clone(),
                            material: my_assets.identifier_connected_material_handle.clone(),
                            transform: transform.with_scale(Vec3::new(1.0, 1.0, 1.0)),
                            ..Default::default()
                        });
                    }
                }

                commands
                    .entity(connection_entity)
                    .insert(Visibility::Visible);
            } else {
                commands
                    .entity(connection_entity)
                    .insert(Visibility::Hidden);
            }
        }
    }
}
