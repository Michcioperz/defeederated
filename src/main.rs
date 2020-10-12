#![feature(never_type)]

use warp::Filter;

pub(crate) mod database;
pub(crate) mod templates;
mod views;

const UPDATE_PERIOD: tokio::time::Duration = tokio::time::Duration::from_secs(5);

#[tokio::main]
async fn main() {
    better_panic::install();
    pretty_env_logger::init();

    let pool = database::open("db.sqlite3").expect("failed to open database");

    let index = warp::get()
        .and(warp::path::end())
        .and(database::with_db(&pool))
        .and_then(views::hello);
    let fetch = warp::get()
        .and(warp::path("objects").and(warp::path::param()))
        .and(database::with_db(&pool))
        .and_then(views::fetch_object);
    let routes = index.or(fetch);

    tokio::spawn(async move {
        periodic_updater(pool).await.unwrap();
    });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn periodic_updater(db: database::Db) -> anyhow::Result<!> {
    let mut interval = tokio::time::interval(UPDATE_PERIOD);
    let client = reqwest::Client::new();
    loop {
        {
            let conn = db.get()?;
            for mut feed in database::list_feeds(conn)? {
                feed.update_from_remote_feed(&client).await?;
            }
        }
        interval.tick().await;
    }
}
