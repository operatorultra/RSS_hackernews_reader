use select::predicate::Name;
use std::error::Error;

use rss::Channel;

use yew::prelude::*;

#[cfg(feature = "ssr")]
async fn fetch_hacker_news_rss() -> Result<Channel, Box<dyn Error>> {
    // https://hnrss.github.io/#firehose-feeds
    // https://hnrss.org/jobs

    let content = reqwest::get("https://hnrss.org/newest?q=ai&points=100&count=25")
        .await?
        .bytes()
        .await?;

    let channel = Channel::read_from(&content[..])?;

    Ok(channel)
}

struct Description {
    article_url: String,
    comments_url: String,
    points: String,
    num_comments: String,
}

fn parse_description(description: &str) -> Description {
    let document = select::document::Document::from(description);

    let article_url = if let Some(a) = document.find(Name("a")).next() {
        a.attr("href").unwrap().to_string()
    } else {
        "".to_string()
    };

    let comments_url = if let Some(a) = document.find(Name("a")).nth(1) {
        a.attr("href").unwrap().to_string()
    } else {
        "".to_string()
    };

    let points = if let Some(p) = document
        .find(Name("p"))
        .filter(|node| node.inner_html().contains("Points:"))
        .next()
    {
        p.text()
    } else {
        "".to_string()
    };

    let num_comments = if let Some(p) = document
        .find(Name("p"))
        .filter(|node| node.inner_html().contains("# Comments:"))
        .next()
    {
        p.text()
    } else {
        "".to_string()
    };

    Description {
        article_url,
        comments_url,
        points,
        num_comments,
    }
}

#[function_component]
fn Content() -> HtmlResult {
    let result = use_prepared_state!(
        async move |_| -> Channel {
            let channel = fetch_hacker_news_rss().await;

            channel.unwrap()
        },
        ()
    )?
    .unwrap();

    let mut items = result.items.clone();

    items.sort_by_key(|item| {
        let res = parse_description(&item.description.as_ref().unwrap().as_str()).points;

        let mut num = 0;
        for word in res.split_whitespace() {
            if let Ok(n) = word.parse::<i32>() {
                num += n;
            }
        }
        num
    });

    items.reverse();

    Ok(html! {
        <>

            <div class="text-center">
            <h1 class="text-2xl p-10">
            {&result.title}
            </h1>
            </div>

            <div class="container mx-auto">
                <ul>
                    {items.iter().map(|item| {
                        let description = parse_description(&item.description.as_ref().unwrap().as_str());
                        html! {

                            <li class="p-5 drop-shadow-md md:drop-shadow-xl bg-slate-100 m-3 rounded-lg">
                            <div class="border-neutral-400">
                            <a href={description.article_url.to_owned()} target="_blank">
                            <h2 class="text-center text-xl hover:font-bold ease-in duration-200">{"☞ "}{ item.title.to_owned() }</h2>
                            </a>
                            <ul>
                            <li>
                            </li>
                            <li>{ format!("Points: {}", description.points) }</li>
                            <li class="py-2">{ description.num_comments }</li>
                            <a class="link_to_article " href={ description.comments_url.to_owned() } target="_blank">
                            <li class="cursor-pointer py-2 bg-sky-500 hover:bg-emerald-400/50 rounded-lg  text-slate-50 text-center ease-in duration-200">
                            { "Check out the comments" }
                            </li>
                            </a>
                            </ul>
                            </div>
                            </li>

                        }
                    }).collect::<Html>()}
                </ul>
            </div>
        </>
    })
}

#[function_component]
pub fn App() -> Html {
    let fallback = html! {<div>{"Loading feed..."}</div>};

    html! {
        <Suspense {fallback}>
            <Content />
        </Suspense>
    }
}
