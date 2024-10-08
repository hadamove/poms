use std::str::FromStr;
use std::sync::{mpsc, Arc, Mutex};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::data::file_loader::AsyncWorkResult;
use crate::app::data::Assembly;

#[derive(Debug, Serialize)]
struct SearchApiRequest<'a> {
    query: Query<'a>,
    return_type: &'a str,
    request_info: RequestInfo<'a>,
    request_options: RequestOptions<'a>,
}

#[derive(Debug, Deserialize)]
struct SearchApiResponse {
    result_set: Vec<String>,
}

#[derive(Clone, Default)]
pub(crate) struct PdbSearchApi {
    client: reqwest::Client,
    /// Remember the last query value to debounce (e.g., to prevent rapid queries)
    last_query_value: Arc<Mutex<Option<String>>>,
}

impl PdbSearchApi {
    const DEBOUNCE_PERIOD_IN_MS: u32 = 1_000;
    const SEARCH_API_URL: &'static str = "https://search.rcsb.org/rcsbsearch/v2/query";
    const MAXIMUM_NUMBER_OF_MATCHES: usize = 20;

    pub(crate) async fn fulltext_search_debounced(
        &self,
        value: &str,
        dispatch: mpsc::Sender<AsyncWorkResult>,
    ) -> anyhow::Result<()> {
        let value = value.to_string();
        {
            let mut last_query_value = self.last_query_value.lock().unwrap();
            *last_query_value = Some(value.clone());
        }

        // Wait for the debounce period
        platform_agnostic_sleep(Self::DEBOUNCE_PERIOD_IN_MS).await;

        // After the delay, check if the query value is still the same
        let should_execute = {
            let last_query_value = self.last_query_value.lock().unwrap();
            last_query_value.as_ref() == Some(&value)
        };

        if !should_execute {
            return Err(anyhow::anyhow!("Query was debounced"));
        }

        let response = self.fulltext_search(&value).await?;
        dispatch
            .send(AsyncWorkResult::SearchResultsParsed {
                result: Self::parse_search_results(response.result_set),
            })
            .ok();

        Ok(())
    }

    async fn fulltext_search(&self, value: &str) -> anyhow::Result<SearchApiResponse> {
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

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(SearchApiResponse {
                result_set: Vec::new(),
            });
        }

        response
            .json::<SearchApiResponse>()
            .await
            .map_err(anyhow::Error::new)
    }

    fn make_uuid() -> String {
        Uuid::new_v4().to_string().replace('-', "")
    }

    fn parse_search_results(results: Vec<String>) -> anyhow::Result<Vec<Assembly>> {
        results
            .into_iter()
            .map(|result_str| Assembly::from_str(&result_str).map_err(anyhow::Error::msg))
            .collect::<anyhow::Result<Vec<Assembly>>>()
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

#[cfg(not(target_arch = "wasm32"))]
async fn platform_agnostic_sleep(duration_in_ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(duration_in_ms as u64));
}

#[cfg(target_arch = "wasm32")]
async fn platform_agnostic_sleep(duration_in_ms: u32) {
    gloo_timers::future::TimeoutFuture::new(duration_in_ms).await;
}
