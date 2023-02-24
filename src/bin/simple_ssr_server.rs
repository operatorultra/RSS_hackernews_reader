// Importing necessary modules
use bytes::Bytes;
use clap::Parser;
use futures::stream::{self, Stream, StreamExt};
use rss_hacker::App;
use std::error::Error;
use std::path::PathBuf;
use warp::Filter;

// Defining a type for boxed errors
type BoxedError = Box<dyn Error + Send + Sync + 'static>;

// Defining a struct to hold command line arguments, uses clap
#[derive(Parser, Debug)]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    /// The "dist" created by trunk directory to be served for hydration.
    dir: PathBuf,
}

// A function to render the HTML content
async fn render(
    index_html_before: String,
    index_html_after: String,
) -> Box<dyn Stream<Item = Result<Bytes, BoxedError>> + Send> {
    // Create a new Yew ServerRenderer
    let renderer = yew::ServerRenderer::<App>::new();

    // Create a stream to return the rendered HTML content
    Box::new(
        stream::once(async move { index_html_before })
            .chain(renderer.render_stream())
            .chain(stream::once(async move { index_html_after }))
            .map(|m| Result::<_, BoxedError>::Ok(m.into())),
    )
}

// Main entry point of the program
#[tokio::main]
async fn main() {
    // Parse command line arguments and get the "dir" option
    let opts = Opt::parse();

    // Read the index.html file into a string
    let index_html_s = tokio::fs::read_to_string(opts.dir.join("index.html"))
        .await
        .expect("failed to read index.html");

    // Split the index.html content into two parts: before and after the <body> tag
    let (index_html_before, index_html_after) = index_html_s.split_once("<body>").unwrap();

    // Convert the before part to a mutable string
    let mut index_html_before = index_html_before.to_owned();

    // Find the end index of the </head> tag
    let head_end_index = index_html_before
        .find("</head>")
        .unwrap_or_else(|| index_html_before.len());

    // Insert a Tailwind CSS script into the </head> tag
    let tailwind_css = r#"<script src="https://cdn.tailwindcss.com"></script>"#;
    index_html_before.insert_str(head_end_index, tailwind_css);

    // Append the <body> tag to the before part
    index_html_before.push_str("<body>");

    // Convert the after part to a mutable string
    let index_html_after = index_html_after.to_owned();

    // Create a filter for the HTML content endpoint
    let html = warp::path::end().then(move || {
        // Clone the before and after parts to be used in the response
        let index_html_before = index_html_before.clone();
        let index_html_after = index_html_after.clone();

        // Call the render function asynchronously and return the result
        async move { warp::reply::html(render(index_html_before, index_html_after).await) }
    });

    // Combine the HTML content filter and the "dir" filter into a single route
    let routes = html.or(warp::fs::dir(opts.dir));

    // Start the server
    println!("You can view the website at: http://localhost:8080/");
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
