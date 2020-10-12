use rusqlite::{params, NO_PARAMS};
use warp::Filter;

pub(crate) type Db = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub(crate) type Conn = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub(crate) fn open<T: AsRef<std::path::Path>>(path: T) -> anyhow::Result<Db> {
    let manager = r2d2_sqlite::SqliteConnectionManager::file(path);
    let pool = r2d2::Pool::new(manager)?;
    create_models(pool.get()?)?;
    Ok(pool)
}

#[derive(Debug)]
pub(crate) struct FeedActor {
    pub(crate) actor_url: String,
    pub(crate) public_key: String,
    pub(crate) private_key: String,
    pub(crate) feed_url: String,
    pub(crate) last_feed_content: String,
}

impl FeedActor {
    pub(crate) fn feed(&self) -> anyhow::Result<feed_rs::model::Feed> {
        let cursor = std::io::Cursor::new(self.last_feed_content.as_bytes());
        match feed_rs::parser::parse(cursor) {
            Ok(feed) => Ok(feed),
            Err(_) => Ok(feed_rs::model::Feed {
                feed_type: feed_rs::model::FeedType::JSON,
                id: self.feed_url.clone(),
                title: None,
                updated: None,
                authors: vec![],
                description: None,
                links: vec![],
                categories: vec![],
                contributors: vec![],
                generator: None,
                icon: None,
                language: None,
                logo: None,
                published: None,
                rights: None,
                ttl: None,
                entries: vec![],
            }),
        }
    }
}

#[derive(Debug)]
pub(crate) struct APObject {
    pub(crate) id: String,
    pub(crate) content: String,
}

pub(crate) fn with_db(
    pool: &Db,
) -> impl warp::Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    let pool = pool.clone();
    warp::any().map(move || pool.clone())
}

pub(crate) fn create_models(conn: Conn) -> anyhow::Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS feed_actors (
            actor_url TEXT PRIMARY KEY,
            public_key TEXT NOT NULL,
            private_key TEXT NOT NULL,
            feed_url TEXT NOT NULL UNIQUE,
            last_feed_content TEXT NOT NULL DEFAULT ''
        )"#,
        NO_PARAMS,
    )?;
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS ap_objects (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL
        )"#,
        NO_PARAMS,
    )?;
    Ok(())
}

pub(crate) fn list_feeds(conn: Conn) -> anyhow::Result<Vec<FeedActor>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            actor_url,
            public_key,
            private_key,
            feed_url,
            last_feed_content
        FROM feed_actors
        "#,
    )?;
    let rows = stmt
        .query_map(rusqlite::NO_PARAMS, |row| {
            Ok(FeedActor {
                actor_url: row.get(0)?,
                public_key: row.get(1)?,
                private_key: row.get(2)?,
                feed_url: row.get(3)?,
                last_feed_content: row.get(4)?,
            })
        })
        .unwrap();
    let results: Vec<rusqlite::Result<FeedActor>> = rows.collect();
    let result: rusqlite::Result<Vec<FeedActor>> = results.into_iter().collect();
    Ok(result?)
}

pub(crate) fn fetch_object<S: AsRef<str>>(conn: Conn, id: S) -> anyhow::Result<APObject> {
    Ok(conn.query_row(
        r#"
        SELECT
            id,
            content
        FROM ap_objects
        WHERE id = ?
        "#,
        params![id.as_ref()],
        |row| {
            Ok(APObject {
                id: row.get(0)?,
                content: row.get(1)?,
            })
        },
    )?)
}
