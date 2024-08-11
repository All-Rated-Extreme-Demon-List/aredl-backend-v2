use uuid::Uuid;

pub struct LeaderboardEntry {
    pub rank: i32,
    pub country_rank: i32,
    pub user_id: Uuid,
    pub country: Option<i32>,
    pub total_points: i32,
    pub pack_points: i32,
    pub hardest: Option<Uuid>,
}

pub struct User {
    pub id: Uuid,
    pub global_name: String,
    pub country: Option<i32>,
}

pub struct Level {
    pub id: Uuid,
    pub name: String,
}

pub struct LeaderboardResolved {
    pub rank: i32,
    pub country_rank: i32,
    pub user: User,
    pub country: Option<i32>,
    pub total_points: i32,
    pub pack_points: i32,
    pub hardest: Level,
}