use actix_web::{guard, web, HttpRequest, Resource, Route};
use std::future::ready;

pub fn redirect_prefix(from: &'static str, to: &'static str) -> Resource {
    web::resource([from.to_owned(), format!("{from}/{{tail:.*}}")]).route(redirect_route(from, to))
}

pub fn redirect_route(from: &'static str, to: &'static str) -> Route {
    web::route()
        .guard(guard::Any(guard::Get()).or(guard::Head()))
        .to(move |req: HttpRequest| {
            let mut location = req.uri().path().replacen(from, to, 1);
            if let Some(query) = req.uri().query() {
                location.push('?');
                location.push_str(query);
            }
            ready(web::Redirect::to(location).permanent())
        })
}

#[cfg(test)]
mod tests;
