use warp::Filter;

pub(crate) mod database;
pub(crate) mod templates;
mod views;

#[tokio::main]
async fn main() {
    better_panic::install();
    pretty_env_logger::init();

    let pool = database::open("db.sqlite3").expect("failed to open database");

    let index = warp::get()
        .and(warp::path::end())
        .and(database::with_db(pool))
        .and_then(views::hello);
    let routes = index;

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
