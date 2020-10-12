use crate::database;
use crate::templates;
use warp::http::Response;

#[derive(Debug)]
struct MiddleErr(anyhow::Error);

impl warp::reject::Reject for MiddleErr {
}

impl From<anyhow::Error> for MiddleErr {
    fn from(err: anyhow::Error) -> MiddleErr {
        MiddleErr(err)
    }
}

impl std::fmt::Display for MiddleErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }

}

impl std::error::Error for MiddleErr {
}

impl From<MiddleErr> for warp::Rejection {
    fn from(err: MiddleErr) -> Self {
        warp::reject::custom(err)
    }
}

pub(crate) async fn hello(db: database::Db) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = db.get().map_err::<anyhow::Error, _>(Into::into).map_err::<MiddleErr, _>(Into::into)?;
    let feeds = database::list_feeds(conn).map_err::<anyhow::Error, _>(Into::into).map_err::<MiddleErr, _>(Into::into)?;
    let html = templates::feed_list(feeds).map_err::<anyhow::Error, _>(Into::into).map_err::<MiddleErr, _>(Into::into)?;
    Ok(warp::reply::html(html.0))
}

pub(crate) async fn fetch_object(
    id: String,
    db: database::Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    let conn = db
        .get()
        .map_err::<anyhow::Error, _>(Into::into)
        .map_err::<MiddleErr, _>(Into::into)?;
    let object = match database::fetch_object(conn, id) {
        Err(e) => {
            return Err(match e.downcast_ref() {
                Some(rusqlite::Error::QueryReturnedNoRows) => warp::reject::not_found(),
                _ => MiddleErr::from(e).into(),
            })
        }
        Ok(obj) => obj,
    };
    Ok(Response::builder()
        .header("content-type", "application/activity+json")
        .body(object.content))
}
