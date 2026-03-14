use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

/// Wrapper om een Askama template als HTML-response terug te geven in Axum.
pub struct HtmlTemplate<T: Template>(pub T);

impl<T: Template> IntoResponse for HtmlTemplate<T> {
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                tracing::error!("Template render fout: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Template render fout").into_response()
            }
        }
    }
}
