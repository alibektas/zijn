use crate::schema::active_sessions;
use diesel::prelude::*;

#[derive(Queryable, Insertable)]
#[diesel(table_name = active_sessions)]
pub struct Session {
    pub id: uuid::Uuid,
    pub session_id: uuid::Uuid,
    pub expires_at: chrono::NaiveDateTime,
}
