pub mod hooks;

mod v001_init;

pub const MIGRATIONS: &[(i32, &str)] = &[
    (23, v001_init::SQL),
];
