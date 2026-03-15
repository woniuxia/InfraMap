pub mod hooks;

mod v001_init;
mod v002_rename_to_systems;
mod v003_topology_node_positions;

pub const MIGRATIONS: &[(i32, &str)] = &[
    (23, v001_init::SQL),
    (24, v002_rename_to_systems::SQL),
    (25, v003_topology_node_positions::SQL),
];
