use crate::database::FeedActor;
use maud::{html, Markup, DOCTYPE};

pub(crate) fn base<S: AsRef<str>>(title: S, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width";
                title { (title.as_ref()) }
            }
            body {
                main {
                    (content)
                }
            }
        }
    }
}

pub(crate) fn feed_list(feeds: Vec<FeedActor>) -> anyhow::Result<Markup> {
    Ok(base(
        "Feeds list",
        html! {
            ul {
                @for feed_actor in feeds {
                    @let feed = feed_actor.last_feed()?;
                    li {
                        a href={(feed_actor.actor_url)} {
                            (feed.title.map(|t| t.content).unwrap_or_else(|| "untitled feed".to_string()))
                        }
                    }
                }
            }
        },
    ))
}
