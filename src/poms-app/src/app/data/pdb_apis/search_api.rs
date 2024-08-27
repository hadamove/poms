use anyhow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct SearchApiRequest<'a> {
    query: Query<'a>,
    return_type: &'a str,
    request_info: RequestInfo<'a>,
    request_options: RequestOptions<'a>,
}

#[derive(Debug, Deserialize)]
pub struct SearchApiResponse {
    pub result_set: Vec<String>,
    pub total_count: usize,
}

pub struct PdbSearchApi {
    client: reqwest::Client,
}

impl PdbSearchApi {
    const SEARCH_API_URL: &'static str = "https://search.rcsb.org/rcsbsearch/v2/query";
    const MAXIMUM_NUMBER_OF_MATCHES: usize = 20;

    pub fn new() -> Self {
        PdbSearchApi {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fulltext_search(&self, value: &str) -> anyhow::Result<SearchApiResponse> {
        let generated_id = Self::make_uuid();

        let params = SearchApiRequest {
            query: Query {
                query_type: "terminal",
                service: "full_text",
                parameters: Parameters { value },
                node_id: 0,
            },
            return_type: "assembly",
            request_info: RequestInfo {
                query_id: &generated_id,
                src: "ui",
            },
            request_options: RequestOptions {
                paginate: Paginate {
                    start: 0,
                    rows: Self::MAXIMUM_NUMBER_OF_MATCHES,
                },
                results_content_type: vec!["experimental"],
                results_verbosity: "compact",
            },
        };

        let response = self
            .client
            .post(Self::SEARCH_API_URL)
            .json(&params)
            .send()
            .await
            .map_err(anyhow::Error::new)?;

        response
            .json::<SearchApiResponse>()
            .await
            .map_err(anyhow::Error::new)
    }

    fn make_uuid() -> String {
        Uuid::new_v4().to_string().replace('-', "")
    }
}

#[derive(Debug, Serialize)]
struct Query<'a> {
    #[serde(rename = "type")]
    query_type: &'a str,
    service: &'a str,
    parameters: Parameters<'a>,
    node_id: i32,
}

#[derive(Debug, Serialize)]
struct Parameters<'a> {
    value: &'a str,
}

#[derive(Debug, Serialize)]
struct RequestInfo<'a> {
    query_id: &'a str,
    src: &'a str,
}

#[derive(Debug, Serialize)]
struct RequestOptions<'a> {
    paginate: Paginate,
    results_content_type: Vec<&'a str>,
    results_verbosity: &'a str,
}

#[derive(Debug, Serialize)]
struct Paginate {
    start: usize,
    rows: usize,
}
