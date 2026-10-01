use std::path::Path;

use dprint_plugin_json::configuration::{ConfigurationBuilder, TrailingCommaKind};
use jsonc_parser::ParseOptions;
use jsonc_parser::cst::{CstContainerNode, CstNode, CstRootNode};

const MAX_CONTAINER_DEPTH: usize = 254;

pub fn format(
    path: &Path,
    input: &str,
    sort_keys: bool,
    indent_width: u8,
) -> Result<String, String> {
    let root =
        CstRootNode::parse(input, &strict_jsonc_options()).map_err(|error| error.to_string())?;

    transform_tree(&root, sort_keys)?;

    let mut config_builder = ConfigurationBuilder::new();
    config_builder
        .indent_width(indent_width)
        .use_tabs(false)
        .package_json_apply_conventions(false)
        .trailing_commas(TrailingCommaKind::Always);
    let config = config_builder.build();
    let transformed = root.to_string();

    dprint_plugin_json::format_text(path, &transformed, &config)
        .map(|formatted| formatted.unwrap_or(transformed))
        .map_err(|error| error.to_string())
}

fn strict_jsonc_options() -> ParseOptions {
    ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
        allow_bare_decimal_point_numbers: false,
        allow_non_finite_numbers: false,
        allow_extended_string_escapes: false,
    }
}

fn transform_tree(root: &CstRootNode, sort_keys: bool) -> Result<(), String> {
    let Some(value) = root.value() else {
        return Ok(());
    };
    let mut pending = vec![(value, 1)];

    while let Some((node, depth)) = pending.pop() {
        let CstNode::Container(container) = node else {
            continue;
        };
        if depth > MAX_CONTAINER_DEPTH {
            return Err(format!(
                "container nesting exceeds the supported maximum of {MAX_CONTAINER_DEPTH}"
            ));
        }

        match container {
            CstContainerNode::Object(object) => {
                let properties = object.properties();
                pending.extend(
                    properties
                        .iter()
                        .filter_map(|property| property.value())
                        .map(|value| (value, depth + 1)),
                );
                if !properties.is_empty() {
                    object.ensure_multiline();
                }
                if sort_keys {
                    object
                        .sort_properties()
                        .by_key(|property| property.decoded_name());
                }
            }
            CstContainerNode::Array(array) => {
                let elements = array.elements();
                pending.extend(elements.iter().cloned().map(|element| (element, depth + 1)));
                if !elements.is_empty() {
                    array.ensure_multiline();
                }
            }
            CstContainerNode::ObjectProp(property) => {
                pending.extend(property.value().map(|value| (value, depth + 1)));
            }
            CstContainerNode::Root(root) => {
                pending.extend(root.value().map(|value| (value, depth + 1)));
            }
        }
    }

    Ok(())
}
