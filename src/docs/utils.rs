use crate::error_handler::ErrorResponse;
use serde_json::json;
use utoipa::openapi::{
    example::ExampleBuilder, path::Operation, Content, ContentBuilder, PathItem, RefOr,
    ResponseBuilder,
};
use utoipa::PartialSchema as _;

pub(super) fn operations(item: &mut PathItem) -> impl Iterator<Item = (&str, &mut Operation)> {
    [
        ("get", &mut item.get),
        ("post", &mut item.post),
        ("put", &mut item.put),
        ("patch", &mut item.patch),
        ("delete", &mut item.delete),
        ("head", &mut item.head),
        ("options", &mut item.options),
        ("trace", &mut item.trace),
        ("query", &mut item.query),
    ]
    .into_iter()
    .filter_map(|(method, operation)| operation.as_mut().map(|operation| (method, operation)))
    .chain(
        item.additional_operations
            .iter_mut()
            .map(|(method, operation)| (method.as_str(), operation)),
    )
}

pub(super) fn add_error_response<'a>(
    operation: &'a mut Operation,
    status: &str,
    description: &str,
) -> Option<&'a mut Content> {
    let response = operation
        .responses
        .responses
        .entry(status.to_owned())
        .or_insert_with(|| ResponseBuilder::new().description("").build().into());
    let RefOr::T(response) = response else {
        return None;
    };
    if !response
        .description
        .split("; ")
        .any(|part| part == description)
    {
        if !response.description.is_empty() {
            response.description.push_str("; ");
        }
        response.description.push_str(description);
    }
    let content = response
        .content
        .entry("application/json".to_owned())
        .or_insert_with(|| {
            ContentBuilder::new()
                .schema(Some(ErrorResponse::schema()))
                .build()
                .into()
        });
    let RefOr::T(content) = content else {
        return None;
    };
    Some(content)
}

pub(super) fn add_error_example(content: &mut Content, name: &str, message: &str) {
    content.examples.entry(name.to_owned()).or_insert_with(|| {
        ExampleBuilder::new()
            .value(Some(json!({"message": message})))
            .build()
            .into()
    });
}
