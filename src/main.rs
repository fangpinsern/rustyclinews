mod theme;

use std::error::Error;
use dotenv::dotenv;

use newsapi::{NewsAPI, Endpoint, Country, Article};

fn render_articles(articles: &Vec<Article>) {
    let theme = theme::default();
    theme.print_text("# Top headline\n\n");
    for a in articles {
        theme.print_text(&format!("`{}`", a.title()));
        theme.print_text(&format!("> *{}*", a.url()));

        theme.print_text("---")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv();

    let api_key = std::env::var("API_KEY")?;

    let mut newsApi = NewsAPI::new(&api_key);

    newsApi
        .endpoint(Endpoint::TopHeadlines)
        .country(Country::Gb);

    let newsApiResponse = newsApi.fetch_async().await?;
    render_articles(&newsApiResponse.articles());
    
    Ok(())
}
