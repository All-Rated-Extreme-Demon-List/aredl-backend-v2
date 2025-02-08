mod schema;
mod error_handler;

use std::{env, fs};
use std::collections::HashMap;
use std::path::Path;
use diesel::{Connection, ExpressionMethods, Insertable, NullableExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use diesel::internal::derives::multiconnection::chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use uuid::Uuid;
use itertools::Itertools;
use crate::error_handler::MigrationError;
use crate::schema::{aredl_levels, aredl_levels_created, aredl_pack_levels, aredl_pack_tiers, aredl_packs, aredl_position_history, aredl_records, roles, user_roles, users, permissions};

type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
type DbConnection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations");

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub user: i64,
    pub link: String,
    pub mobile: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub id: i32,
    pub name: String,
	pub description: Option<String>,
    pub author: i64,
    pub creators: Vec<i64>,
    pub verifier: i64,
    pub verification: String,
    pub song: Option<i32>,
    pub tags: Option<Vec<Option<String>>>,   
    pub records: Vec<Record>,
}

#[derive(Serialize, Deserialize)]
pub struct Pack {
    pub name: String,
    pub levels: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PackTier {
    pub name: String,
    pub color: String,
    pub packs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RoleMember {
    pub name: i64,
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
    pub id: Option<Uuid>,
    pub username: String,
    pub json_id: i64,
    pub global_name: String,
    pub placeholder: bool,
    pub ban_level: i32,
    pub country: Option<i32>,
    pub discord_id: Option<String>
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[diesel(table_name=aredl_levels)]
pub struct LevelCreate {
    pub id: Option<Uuid>,
    pub position: i32,
    pub name: String,
	pub description: Option<String>,
    pub publisher_id: Uuid,
    pub legacy: bool,
    pub level_id: i32,
    pub two_player: bool,
	pub tags: Option<Vec<Option<String>>>,
    pub song: Option<i32>,
}

pub struct LevelInfo {
    pub legacy: bool,
    pub two_player: bool,
    pub original_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Country {
    code: i32,
    name: String,
    users: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangelogEntry {
    pub date: i64,
    pub action: String,
    pub name: String,
    pub to_rank: Option<i32>,
    pub from_rank: Option<i32>,
    pub above: Option<String>,
    pub below: Option<String>
}

#[derive(Insertable, Debug)]
#[diesel(table_name=aredl_position_history)]
pub struct ChangelogResolved {
    pub new_position: Option<i32>,
    pub old_position: Option<i32>,
    pub legacy: Option<bool>,
    pub affected_level: Uuid,
    pub level_above: Option<Uuid>,
    pub level_below: Option<Uuid>,
    pub created_at: NaiveDateTime
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
        ("owner", ("owner", 110)),
        ("dev", ("developer", 100)),
        ("admin", ("admin", 90)),
        ("mod", ("mod", 50)),
        ("helper", ("helper", 30)),
        ("patreon", ("plus", 5)),
    ]);

    let permissions_data = vec![
        ("record_modify", 20),
        ("placeholder_create", 25),
        ("user_modify", 25),
        ("pack_tier_modify", 30),
        ("pack_modify", 40),
        ("level_modify", 50),
        ("merge_review", 60),
        ("user_ban", 85),
        ("direct_merge", 90),
        ("role_manage", 100)
    ];

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

    let pack_tier_data = load_json_from_file::<Vec<PackTier>>(
        aredl_path.join("_packtiers.json").as_path());

    let list_init = load_json_from_file::<Vec<String>>(
        aredl_path.join("_list_init.json").as_path());

    let list_legacy_init = load_json_from_file::<Vec<String>>(
        aredl_path.join("_legacy_init.json").as_path());

    let changelog = load_json_from_file::<Vec<ChangelogEntry>>(
        aredl_path.join("_changelog.json").as_path());

    let country_data = load_json_from_file::<Vec<Country>>(
        aredl_path.join("_countries.json").as_path());

    let name_map = load_json_from_file::<HashMap<i64, String>>(
        aredl_path.join("_name_map.json").as_path());

    let discord_map = load_json_from_file::<HashMap<i64, String>>(
        aredl_path.join("_discord_ids.json").as_path());

    let user_country_map: HashMap<i64, i32> = country_data
        .into_iter()
        .flat_map(|country|
            country.users.into_iter().map(move |user| (user, country.code))
        )
        .collect();

    let banned_users = load_json_from_file::<Vec<i64>>(
        aredl_path.join("_leaderboard_banned.json").as_path());

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
        .unique_by(|id| id.clone())
        .map(|id| {
            let country = user_country_map.get(&id).cloned();
            let username = name_map.get(&id).cloned().expect(format!("Username not found for id: {}", id).as_str());
            CreateUser {
                id: None,
                username: username.clone(),
                json_id: id,
                global_name: username,
                placeholder: true,
                ban_level: if banned_users.contains(&id) { 1 } else { 0 },
                country,
                discord_id: discord_map.get(&id).cloned(),
            }
        })
        .collect();

    println!("\tUsers found: {}", &users.len());

    let pack_tier_name_map: HashMap<String, Vec<String>> = pack_tier_data
        .iter()
        .map(|tier| (tier.name.clone(), tier.packs.clone()))
        .collect();

    println!("Migrating");
    db_conn.transaction::<_, MigrationError, _>(|conn| {
        println!("\tLoading user and level id's");
        let old_user_ids: HashMap<i64, Uuid> = users::table
            .filter(users::json_id.is_not_null())
            .select((
            users::json_id.assume_not_null(),
            users::id,
            ))
            .load::<(i64, Uuid)>(conn)?
            .into_iter()
            .collect();

        let users = users.into_iter()
            .map(|mut user| {
                user.id = old_user_ids.get(&user.json_id).map(|id| id.clone());
                user
            })
            .collect_vec();

        let old_level_ids: HashMap<String, Uuid> = aredl_levels::table.select((
            aredl_levels::name,
            aredl_levels::id
        ))
            .load::<(String, Uuid)>(conn)?
            .into_iter()
            .collect();

        println!("\tResetting db");
        conn.revert_all_migrations(MIGRATIONS)?;
        conn.run_pending_migrations(MIGRATIONS)?;

        println!("\tInserting users");
        diesel::insert_into(users::table)
            .values(&users)
            .execute(conn)?;

        let user_map: HashMap<i64, Uuid> = users::table
            .filter(users::json_id.is_not_null())
            .select((
            users::json_id.assume_not_null(),
            users::id,
            )).load::<(i64, Uuid)>(conn)?
            .into_iter()
            .collect();

        println!("\tLoading levels");
        let level_insert: Vec<LevelCreate> = levels.iter()
            .enumerate()
            .map(|(position, (level_data, level_data_ext))| LevelCreate {
                id: old_level_ids.get(&level_data.name).map(|id| id.clone()),
                position: (position + 1) as i32,
                name: level_data.name.clone(),
				description: level_data.description.clone(),
                publisher_id: user_map.get(&level_data.author).unwrap().clone(),
                legacy: level_data_ext.legacy,
                level_id: level_data.id,
                two_player: level_data_ext.two_player,
				tags: level_data.tags.clone(),
                song: level_data.song,
            })
            .collect();

        println!("\tInserting levels");

        diesel::sql_query("ALTER TABLE aredl_levels DISABLE TRIGGER aredl_level_place")
            .execute(conn)?;

        diesel::sql_query("ALTER TABLE aredl_levels DISABLE TRIGGER aredl_level_place_history")
            .execute(conn)?;

        diesel::insert_into(aredl_levels::table)
            .values(&level_insert)
            .execute(conn)?;

        diesel::sql_query("ALTER TABLE aredl_levels ENABLE TRIGGER aredl_level_place")
            .execute(conn)?;

        diesel::sql_query("ALTER TABLE aredl_levels ENABLE TRIGGER aredl_level_place_history")
            .execute(conn)?;

        let level_map: HashMap<(i32, bool), Uuid> = aredl_levels::table
            .select((
                aredl_levels::id,
                aredl_levels::level_id,
                aredl_levels::two_player)
            ).load::<(Uuid, i32, bool)>(conn)?
            .iter().map(|(id, level_id, two_player)| ((*level_id, *two_player), *id))
            .collect();

        println!("\tInserting pack-tiers");

        diesel::insert_into(aredl_pack_tiers::table)
            .values(
                pack_tier_data.iter().enumerate().map(|(index, tier)| (
                    aredl_pack_tiers::name.eq(&tier.name),
                    aredl_pack_tiers::color.eq(&tier.color),
                    aredl_pack_tiers::placement.eq(index as i32),
                    )).collect::<Vec<_>>()
            ).execute(conn)?;

        let pack_tier_map: HashMap<String, Uuid> = aredl_pack_tiers::table
            .select((
                aredl_pack_tiers::name,
                aredl_pack_tiers::id,
                ))
            .load::<(String, Uuid)>(conn)?
            .iter()
            .flat_map(|(name, id)| pack_tier_name_map.get(name).unwrap().into_iter().map(|pack| (pack.clone(), id.clone())))
            .collect();

        println!("\tInserting packs");
        diesel::insert_into(aredl_packs::table)
            .values(
                pack_data.iter().map(|pack| (
                    aredl_packs::name.eq(&pack.name),
                    aredl_packs::tier.eq(pack_tier_map.get(&pack.name).unwrap())
                )).collect::<Vec<_>>()
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
                        aredl_pack_levels::level_id.eq(level_map.get(&(*level_id_map.get(level_name).unwrap(), level_name.ends_with("2p"))).unwrap()),
                        aredl_pack_levels::pack_id.eq(pack_map.get(&pack.name).unwrap())
                        )
                    )
                ).collect::<Vec<_>>()
            ).execute(conn)?;

        println!("\tInserting changelog");
        let init_levels = list_init.into_iter().map(|name|
            (level_map.get(&(*level_id_map.get(&name).unwrap(), name.ends_with("2p"))).unwrap().clone(), false)
        ).chain(
            list_legacy_init.into_iter().map(|name|
                (level_map.get(&(*level_id_map.get(&name).unwrap(), name.ends_with("2p"))).unwrap().clone(), true)
            )
        ).collect::<Vec<(Uuid, bool)>>();

        let first_timestamp = changelog.first().map(|entry| entry.date).unwrap_or(1701030687);

        let init_changelog = init_levels.iter().enumerate().map(|(position, (id, legacy))| {
            let above = if position > 0 {
                Some(init_levels[position - 1].0)
            } else {
                None
            };
            let below = if position < init_levels.len() - 1 {
                Some(init_levels[position + 1].0)
            } else {
                None
            };
            (id, legacy, above, below);
            ChangelogResolved {
                new_position: Some((position + 1) as i32),
                old_position: None,
                legacy: Some(legacy.clone()),
                affected_level: id.clone(),
                level_above: above,
                level_below: below,
                created_at: DateTime::from_timestamp(first_timestamp, 0).unwrap().naive_utc(),
            }
        });

        let changelog_data = init_changelog.chain(
            changelog.into_iter().map(|entry| {
                let legacy = match entry.action.as_str() {
                    "tolegacy" => Some(true),
                    "fromlegacy" => Some(false),
                    "placed" => Some(false),
                    _ => None
                };
                ChangelogResolved {
                    new_position: entry.to_rank,
                    old_position: entry.from_rank,
                    legacy,
                    affected_level: level_map.get(&(*level_id_map.get(&entry.name).unwrap(), entry.name.ends_with("2p"))).unwrap().clone(),
                    level_above: entry.above.map(|name| level_map.get(&(*level_id_map.get(&name).unwrap(), name.ends_with("2p"))).unwrap().clone()),
                    level_below: entry.below.map(|name| level_map.get(&(*level_id_map.get(&name).unwrap(), name.ends_with("2p"))).unwrap().clone()),
                    created_at: DateTime::from_timestamp(entry.date, 0).unwrap().naive_utc(),
                }
            })
        ).collect::<Vec<ChangelogResolved>>();

        diesel::insert_into(aredl_position_history::table)
            .values(changelog_data)
            .execute(conn)?;

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
                                user_map.get(creator).unwrap()
                            )
                        )
                    )
                ).collect::<Vec<_>>()
            ).execute(conn)?;

        println!("\tInserting records");

        let now = Utc::now().naive_utc();

        let records = levels.iter().map(|(level,level_data_ext)|
            (
                aredl_records::level_id.eq(level_map.get(&(level.id, level_data_ext.two_player)).unwrap()),
                aredl_records::submitted_by.eq(user_map.get(&level.verifier).unwrap()),
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
                        aredl_records::submitted_by.eq(user_map.get(&record.user).unwrap()),
                        aredl_records::mobile.eq(record.mobile.unwrap_or(false)),
                        aredl_records::video_url.eq(&record.link),
                        aredl_records::created_at.eq(created_at),
                    )
                })
            )
        ).collect::<Vec<_>>();

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
                roles::role_desc,
                roles::id)
            ).load::<(String, i32)>(conn)?
            .into_iter().collect();

        diesel::insert_into(user_roles::table)
            .values(
                role_data.iter()
                    .flat_map(|role_list|
                        role_list.members.iter().map(|member| (
                            user_roles::user_id.eq(user_map.get(&member.name).unwrap()),
                            user_roles::role_id.eq(role_map.get(
                                roles_map.get(&role_list.role.as_str()).unwrap().0
                            ).unwrap())
                            ))
                    ).collect::<Vec<_>>()
            ).execute(conn)?;

        println!("\tInserting permissions");
        diesel::insert_into(permissions::table)
            .values(
                permissions_data
                    .iter()
                    .map(|(permission, privilege_level)| {
                        (
                            permissions::permission.eq(*permission),
                            permissions::privilege_level.eq(*privilege_level),
                        )
                    })
                    .collect::<Vec<_>>(),
            ).execute(conn)?;

        Ok(())
    }).expect("Failed to migrate");

    println!("\tUpdating materialized views");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl_user_leaderboard")
        .execute(&mut db_conn).expect("Failed to update leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl_country_leaderboard")
        .execute(&mut db_conn).expect("Failed to update country leaderboard");

    diesel::sql_query("REFRESH MATERIALIZED VIEW aredl_position_history_full_view")
        .execute(&mut db_conn).expect("Failed to update position history");

}
