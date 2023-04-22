use std::fs::read_to_string;

use crate::search_results_handler::aggregator::aggregate;
use actix_web::{get, web, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: Option<String>,
    page: Option<u32>,
}

#[get("/")]
pub async fn index(
    hbs: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = hbs.render("index", &"").unwrap();
    Ok(HttpResponse::Ok().body(page_content))
}

pub async fn not_found(
    hbs: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = hbs.render("404", &"")?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page_content))
}

#[get("/search")]
pub async fn search(
    hbs: web::Data<Handlebars<'_>>,
    req: HttpRequest,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let params = web::Query::<SearchParams>::from_query(req.query_string())?;
    match &params.q {
        Some(query) => {
            if query.trim().is_empty() {
                Ok(HttpResponse::Found()
                    .insert_header(("location", "/"))
                    .finish())
            } else {
                let results_json: crate::search_results_handler::aggregation_models::SearchResults =
                    aggregate(query, params.page).await?;
                let page_content: String = hbs.render("search", &results_json)?;
                Ok(HttpResponse::Ok().body(page_content))
            }
        }
        None => Ok(HttpResponse::Found()
            .insert_header(("location", "/"))
            .finish()),
    }
}

#[get("/robots.txt")]
pub async fn robots_data(_req: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = read_to_string("./public/robots.txt")?;
    Ok(HttpResponse::Ok()
        .content_type("text/plain; charset=ascii")
        .body(page_content))
}

#[get("/about")]
pub async fn about(
    hbs: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = hbs.render("about", &"")?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[get("/settings")]
pub async fn settings(
    hbs: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let page_content: String = hbs.render("settings", &"")?;
    Ok(HttpResponse::Ok().body(page_content))
}
