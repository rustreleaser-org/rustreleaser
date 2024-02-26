use super::github_client::GITHUB_TOKEN;
use reqwest::{
    header::{ACCEPT, USER_AGENT},
    RequestBuilder,
};

pub trait Headers {
    fn default_headers(self) -> RequestBuilder;
}

impl Headers for RequestBuilder {
    fn default_headers(self) -> RequestBuilder {
        self.bearer_auth(GITHUB_TOKEN.to_string())
            .header(ACCEPT, "application/vnd.github.VERSION.sha")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(USER_AGENT, "rustreleaser")
    }
}

#[macro_export]
macro_rules! put {
    ($url:expr, $body:expr) => {{
        use $crate::{github::macros::Headers, http::ResponseHandler};

        $crate::http::HttpClient::new()
            .put($url)
            .default_headers()
            .body($body)
            .send()
            .await
            .handle()
            .await
    }};
}

#[macro_export]
macro_rules! get {
    ($url:expr) => {{
        use $crate::{github::macros::Headers, http::ResponseHandler};

        $crate::http::HttpClient::new()
            .get($url)
            .default_headers()
            .send()
            .await
            .handle()
            .await
    }};
}

#[macro_export]
macro_rules! post {
    ($url:expr, $body:expr) => {{
        use $crate::{github::macros::Headers, http::ResponseHandler};

        $crate::http::HttpClient::new()
            .post($url)
            .default_headers()
            .body($body)
            .send()
            .await
            .handle()
            .await
    }};
}

#[macro_export]
macro_rules! form {
    ($url:expr, $form:expr) => {{
        use reqwest::header::CONTENT_TYPE;
        use $crate::{github::macros::Headers, http::ResponseHandler};

        $crate::http::HttpClient::new()
            .post($url)
            .default_headers()
            .header(CONTENT_TYPE, "application/octet-stream")
            .multipart($form)
            .send()
            .await
            .handle()
            .await
    }};
}
