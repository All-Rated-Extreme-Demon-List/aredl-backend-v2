mod schema;
mod error_handler;

use std::{env, fs};
use std::collections::{HashMap};
use std::path::Path;
use diesel::{Connection, ExpressionMethods, Insertable, PgConnection, QueryDsl, RunQueryDsl};
use diesel::internal::derives::multiconnection::chrono::{Duration, Utc};
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;
use itertools::Itertools;
use crate::error_handler::MigrationError;
use crate::schema::{aredl_levels, aredl_levels_created, aredl_pack_levels, aredl_packs, aredl_records, roles, user_roles, users};

type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
type DbConnection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations");

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub user: String,
    pub link: String,
    pub mobile: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub id: i32,
    pub name: String,
    pub author: String,
    pub creators: Vec<String>,
    pub verifier: String,
    pub verification: String,
    pub records: Vec<Record>,
}

#[derive(Serialize, Deserialize)]
pub struct Pack {
    pub name: String,
    pub levels: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RoleMember {
    pub name: String,
    pub link: String,
}

#[derive(Serialize, Deserialize)]
pub struct RoleList {
    pub role: String,
    pub members: Vec<RoleMember>,
}

#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name=users)]
pub struct CreateUser {
    pub username: String,
    pub global_name: String,
    pub placeholder: bool,
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[diesel(table_name=aredl_levels)]
pub struct LevelCreate {
    pub position: i32,
    pub name: String,
    pub publisher_id: Uuid,
    pub legacy: bool,
    pub level_id: i32,
    pub two_player: bool,
}

pub struct LevelInfo {
    pub legacy: bool,
    pub two_player: bool,
    pub original_name: String,
}

pub fn load_json_from_file<T>(path: &Path) -> T
where T: DeserializeOwned,
{
    let file = fs::File::open(path)
        .expect(format!("Failed to open file {}", path.to_str().unwrap()).as_str());
    let json: T = serde_json::from_reader(file)
        .expect("Failed to parse json");
    json
}

fn main() {
    dotenv().ok();

    let roles_map: HashMap<&str, (&str, i32)> = HashMap::from([
        ("owner", ("owner", 100)),
        ("coowner", ("coowner", 90)),
        ("dev", ("developer", 80)),
        ("admin", ("admin", 70)),
        ("trial", ("mod", 50)),
        ("helper", ("helper", 30)),
        ("patreon", ("plus", 5)),
    ]);

    let db_url = env::var("DATABASE_URL").expect("Database url not set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let mut db_conn: DbConnection = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create db pool")
        .get().unwrap();

    println!("Loading data");
    let aredl_path_str = env::var("AREDL_DATA_PATH").expect("AREDL_DATA_PATH not set");

    let aredl_path = Path::new(&aredl_path_str);

    let level_names = load_json_from_file::<Vec<String>>(
        aredl_path.join("_list.json").as_path());

    let legacy_names = load_json_from_file::<Vec<String>>(
        aredl_path.join("_legacy.json").as_path());

    let mut role_data = load_json_from_file::<Vec<RoleList>>(
        aredl_path.join("_editors.json").as_path());

    let role_data_supporters = load_json_from_file::<Vec<RoleList>>(
        aredl_path.join("_supporters.json").as_path());

    let pack_data = load_json_from_file::<Vec<Pack>>(
        aredl_path.join("_packlist.json").as_path());

    role_data.extend(role_data_supporters);

    let levels: Vec<(Level, LevelInfo)> = level_names
        .iter()
        .map(|name| (name, false))
        .chain(
            legacy_names
                .iter()
                .map(|name| (name, true))
        )
        .map(|(name, legacy)|
            (load_json_from_file::<Level>(
                aredl_path.join(format!("{}.json", name).as_str()).as_path()
            ),
                LevelInfo {
                    legacy,
                    two_player: name.ends_with("2p"),
                    original_name: name.clone(),
                }
             )
        ).collect();

    let level_id_map: HashMap<String, i32> = levels
        .iter()
        .map(|(level, level_ext)| (level_ext.original_name.clone(), level.id))
        .collect();

    println!("\tLevels found: {}", &levels.len());

    let users: Vec<CreateUser> = levels
        .iter()
        .map(|(level, _)| level.author.clone())
        .chain(
            levels.iter().map(|(level, _)| level.verifier.clone())
        )
        .chain(
            levels.iter().flat_map(|(level, _)| level.creators.clone())
        )
        .chain(
            levels.iter().flat_map(|(level, _)| level.records.iter().clone()).map(|record| record.user.clone())
        )
        .chain(
            role_data.iter().flat_map(|data| data.members.iter().map(|member| member.name.clone()))
        )
        .unique_by(|name| name.to_lowercase())
        .map(|name| CreateUser {
            username: name.clone(),
            global_name: name,
            placeholder: true,
        })
        .collect();

    println!("\tUsers found: {}", &users.len());

    println!("Migrating");
    db_conn.transaction::<_, MigrationError, _>(|conn| {
        println!("\tResetting db");
        conn.revert_all_migrations(MIGRATIONS)?;
        conn.run_pending_migrations(MIGRATIONS)?;

        println!("\tInserting users");
        diesel::insert_into(users::table)
            .values(&users)
            .execute(conn)?;

        let user_map: HashMap<String, Uuid> = users::table.select((
            users::id,
            users::username,
            )).load::<(Uuid, String)>(conn)?
            .iter()
            .map(|(id, name)| (name.to_lowercase(), id.clone()))
            .collect();

        println!("\tLoading levels");
        let level_insert: Vec<LevelCreate> = levels.iter()
            .enumerate()
            .map(|(position, (level_data, level_data_ext))| LevelCreate {
                position: (position + 1) as i32,
                name: level_data.name.clone(),
                publisher_id: *user_map.get(&level_data.author.to_lowercase()).unwrap(),
                legacy: level_data_ext.legacy,
                level_id: level_data.id,
                two_player: level_data_ext.two_player,
            })
            .collect();

        println!("\tInserting levels");

        diesel::sql_query("ALTER TABLE aredl_levels DISABLE TRIGGER aredl_level_place")
            .execute(conn)?;

        diesel::insert_into(aredl_levels::table)
            .values(&level_insert)
            .execute(conn)?;

        diesel::sql_query("ALTER TABLE aredl_levels ENABLE TRIGGER aredl_level_place")
            .execute(conn)?;

        let level_map: HashMap<(i32, bool), Uuid> = aredl_levels::table
            .select((
                aredl_levels::id,
                aredl_levels::level_id,
                aredl_levels::two_player)
            ).load::<(Uuid, i32, bool)>(conn)?
            .iter().map(|(id, level_id, two_player)| ((*level_id, *two_player), *id))
            .collect();

        println!("\tInserting packs");
        diesel::insert_into(aredl_packs::table)
            .values(
                pack_data.iter().map(|pack| aredl_packs::name.eq(&pack.name)).collect::<Vec<_>>()
            ).execute(conn)?;

        let pack_map: HashMap<String, Uuid> = aredl_packs::table
            .select((
                aredl_packs::name,
                aredl_packs::id)
            ).load::<(String, Uuid)>(conn)?
            .iter()
            .map(|(name, id)| (name.clone(), *id))
            .collect();

        diesel::insert_into(aredl_pack_levels::table)
            .values(
                pack_data.iter().flat_map(|pack|
                    pack.levels.iter().map(|level_name|(
                        aredl_pack_levels::level_id.eq(level_map.get(&(*level_id_map.get(level_name).unwrap(), false)).unwrap()),
                        aredl_pack_levels::pack_id.eq(pack_map.get(&pack.name).unwrap())
                        )
                    )
                ).collect::<Vec<_>>()
            ).execute(conn)?;

        println!("\tInserting creators");
        diesel::insert_into(aredl_levels_created::table)
            .values(
                levels.iter().flat_map(|(level,level_data_ext)|
                    level.creators.iter().map(|creator|
                        (
                            aredl_levels_created::level_id.eq(
                                level_map.get(&(level.id, level_data_ext.two_player)).unwrap()
                            ),
                            aredl_levels_created::user_id.eq(
                                user_map.get(&creator.to_lowercase()).unwrap()
                            )
                        )
                    )
                ).collect::<Vec<_>>()
            ).execute(conn)?;

        let now = Utc::now().naive_utc();

        let records = levels.iter().map(|(level,level_data_ext)|
            (
                aredl_records::level_id.eq(level_map.get(&(level.id, level_data_ext.two_player)).unwrap()),
                aredl_records::submitted_by.eq(user_map.get(&level.verifier.to_lowercase()).unwrap()),
                aredl_records::mobile.eq(false),
                aredl_records::video_url.eq(&level.verification),
                aredl_records::created_at.eq(now),
            )
        ).chain(
            levels.iter().flat_map(|(level,level_data_ext)|
                level.records.iter().enumerate().map(|(index, record)| {
                    let created_at = now + Duration::seconds((index + 1) as i64);
                    (
                        aredl_records::level_id.eq(level_map.get(&(level.id, level_data_ext.two_player)).unwrap()),
                        aredl_records::submitted_by.eq(user_map.get(&record.user.to_lowercase()).unwrap()),
                        aredl_records::mobile.eq(record.mobile.unwrap_or(false)),
                        aredl_records::video_url.eq(&record.link),
                        aredl_records::created_at.eq(created_at),
                    )
                })
            )
        ).collect::<Vec<_>>();

        println!("\tInserting records");
        for record_chunk in records.chunks(4000) {
            diesel::insert_into(aredl_records::table)
                .values(record_chunk)
                .execute(conn)?;
        }

        println!("\tInserting roles");
        diesel::insert_into(roles::table)
            .values(
                role_data.iter()
                    .map(|role_list| *roles_map.get(&role_list.role.as_str()).unwrap())
                    .map(|(role_name, privilege_level)| (
                        roles::privilege_level.eq(privilege_level),
                        roles::role_desc.eq(role_name)
                    )).collect::<Vec<_>>()
            ).execute(conn)?;

        let role_map: HashMap<String, i32> = roles::table
            .select((
                roles::id,
                roles::role_desc)
            ).load::<(i32, String)>(conn)?
            .iter().map(|(id, name)| (name.clone(), *id))
            .collect();

        diesel::insert_into(user_roles::table)
            .values(
                role_data.iter()
                    .flat_map(|role_list|
                        role_list.members.iter().map(|member| (
                            user_roles::user_id.eq(user_map.get(&member.name.to_lowercase()).unwrap()),
                            user_roles::role_id.eq(role_map.get(
                                roles_map.get(&role_list.role.as_str()).unwrap().0
                            ).unwrap())
                            ))
                    ).collect::<Vec<_>>()
            ).execute(conn)?;

        Ok(())
    }).expect("Failed to migrate");
}
