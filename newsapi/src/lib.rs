use thiserror::Error;
use url::Url;
use serde::Deserialize;

#[cfg(feature = "async")]
use reqwest;


#[derive(Deserialize, Debug)]
pub struct NewsAPIResponse {
    status: String,
    code: Option<String>,
    pub articles: Vec<Article>,
}

impl NewsAPIResponse {
    pub fn articles(&self) -> &Vec<Article> {
        &self.articles
    }
}

#[derive(Deserialize, Debug)]
pub struct Article {
    title: String,
    url: String,
    description: Option<String>
}

impl Article {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn desc(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

const BASE_URL: &str = "https://newsapi.org/v2";

#[derive(Error, Debug)]
pub enum NewsApiError {
    #[error("Failed fetching articles")]
    RequestFailed(#[from] ureq::Error),
    #[error("Failed converting response to string")]
    FailedResponseToString(#[from] std::io::Error),
    #[error("Article parsing failed")]
    ArticleParseFailed(#[from] serde_json::Error),
    #[error("URL parsing failed")]
    UrlParsingFailed(#[from] url::ParseError),
    #[error("Request failed: {0}")]
    BadRequest(&'static str),
    #[error("Async request failed")]
    #[cfg(feature = "async")]
    AsyncRequestFailed(#[from] reqwest::Error)
}

pub enum Endpoint {
    TopHeadlines
}

impl ToString for Endpoint {
    fn to_string(&self) -> String {
        match self {
            Self::TopHeadlines => "top-headlines".to_string()
        }
    }
}

pub enum Country {
    Us,
    Sg,
    Gb
}

impl ToString for Country {
    fn to_string(&self) -> String {
        match self {
            Self::Us => "us".to_string(),
            Self::Sg => "sg".to_string(),
            Self::Gb => "gb".to_string()
        }
    }
}

pub struct NewsAPI {
    api_key: String,
    endpoint: Endpoint,
    country: Country
}

impl NewsAPI {
    pub fn new(api_key: &str) -> NewsAPI {
        NewsAPI{
            api_key: api_key.to_string(),
            endpoint: Endpoint::TopHeadlines,
            country: Country::Us
        }
    }

    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut NewsAPI {
        self.endpoint = endpoint;
        return self
    } 

    pub fn country(&mut self, country: Country) -> &mut NewsAPI {
        self.country = country;
        return self
    }

    fn prepare_url(&self) -> Result<String, NewsApiError> {
        let mut url = Url::parse(BASE_URL)?;

        url.path_segments_mut().unwrap().push(&self.endpoint.to_string());

        let country = format!("country={}", self.country.to_string());
        url.set_query(Some(&country));

        Ok(url.to_string())
    }

    pub fn fetch(&self) -> Result<NewsAPIResponse, NewsApiError> {
        let url = self.prepare_url()?;
        let req = ureq::get(&url).set("Authorization", &self.api_key);
        let res: NewsAPIResponse = req.call()?.into_json()?;

        match res.status.as_str() {
            "ok" => return Ok(res),
            _ => return Err(map_response_err(res.code))
        }
    }

    #[cfg(feature = "async")]
    pub async fn fetch_async(&self) -> Result<NewsAPIResponse, NewsApiError> {
        let url = self.prepare_url()?;
        dbg!(url.to_string());
        let client = reqwest::Client::new();
        let req = client
            .request(reqwest::Method::GET, url)
            .header("Authorization", &self.api_key)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| NewsApiError::AsyncRequestFailed(e))?;

        
        /*
        It might be a good idea to split response and decoding up.
        Here there was an error that caused the correct fields to be wrong.
        Unsure why but when you choose different countries, it breaks.

        When using the debugging method in this link:
        https://stackoverflow.com/questions/65757876/how-to-fix-reqwest-json-decode-errors

        The following error was found:
        &res = "{
                    \"status\":\"error\",
                    \"code\":\"userAgentMissing\",
                    \"message\":\"Please set your User-Agent header to identify your application. Anonymous requests are not allowed.\"
        }"
        
        Hence found that is was due to missing user agent? 
        But abit weird. Why does it work of one country but not the others?
        */
        let res = client
            .execute(req)
            .await?
            .text()
            .await
            .map_err(|e| NewsApiError::AsyncRequestFailed(e))?;

        let resDecoded: NewsAPIResponse = serde_json::from_str(&res)?;

        match resDecoded.status.as_str() {
            "ok" => return Ok(resDecoded),
            _ => return Err(map_response_err(resDecoded.code))
        }
    }
}


fn map_response_err(code: Option<String>) -> NewsApiError {

    if let Some(code) = code {
        match code.as_str() {
            "apiKeyDisabled" => NewsApiError::BadRequest("Your API key has been disabled"),
            _ => NewsApiError::BadRequest("Unknown error")
        }
    } else {
        NewsApiError::BadRequest("Unknown error")
    }
}