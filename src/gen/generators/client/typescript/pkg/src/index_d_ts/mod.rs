mod docs;

use inflector::Inflector;
use crate::core::action::{ResMeta, ResData, Action, UPDATE_HANDLER, CREATE_HANDLER, FIND_FIRST_HANDLER, UPSERT_HANDLER, UPDATE_MANY_HANDLER};
use crate::core::field::r#type::FieldTypeOwner;
use crate::gen::generators::client::typescript::pkg::src::index_d_ts::docs::{action_doc, action_group_doc, create_or_update_doc, credentials_doc, cursor_doc, field_doc, include_doc, main_object_doc, nested_connect_doc, nested_create_doc, nested_create_or_connect_doc, nested_delete_doc, nested_disconnect_doc, nested_set_doc, nested_update_doc, nested_upsert_doc, order_by_doc, page_number_doc, page_size_doc, relation_doc, select_doc, skip_doc, take_doc, unique_connect_create_doc, unique_connect_doc, unique_where_doc, where_doc, where_doc_first, with_token_doc};
use crate::gen::generators::client::typescript::r#type::ToTypeScriptType;
use crate::core::graph::Graph;
use crate::core::model::{Model};
use crate::core::model::index::ModelIndexType::{Primary, Unique};
use crate::gen::generators::client::typescript::pkg::src::filter_d_ts::generate_filter_types;
use crate::gen::generators::client::typescript::pkg::src::operation_d_ts::generate_operation_types;
use crate::gen::generators::client::typescript::pkg::src::runtime_d_ts::generate_client_runtime_types;
use crate::gen::internal::code::Code;
use crate::gen::generators::server::nodejs::runtime_d_ts::generate_server_runtime_types;

fn generate_model_create_nested_input(_graph: &Graph, model: &Model, without: Option<&str>, many: bool) -> String {
    let model_name = model.name();
    let without_title = if let Some(title) = without {
        let title = title.to_pascal_case();
        format!("Without{title}")
    } else {
        "".to_owned()
    };
    let many_title = if many { "Many" } else { "One" };
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}CreateNested{many_title}{without_title}Input = {{"), |b| {
            if many {
                b.doc(nested_create_doc(model, many));
                b.line(format!("create?: Enumerable<{model_name}Create{without_title}Input>"));
                b.doc(nested_create_or_connect_doc(model, many));
                b.line(format!("connectOrCreate?: Enumerable<{model_name}ConnectOrCreate{without_title}Input>"));
                b.doc(nested_connect_doc(model, many));
                b.line(format!("connect?: Enumerable<{model_name}WhereUniqueInput>"));
            } else {
                b.doc(nested_create_doc(model, many));
                b.line(format!("create?: {model_name}Create{without_title}Input"));
                b.doc(nested_create_or_connect_doc(model, many));
                b.line(format!("connectOrCreate?: {model_name}ConnectOrCreate{without_title}Input"));
                b.doc(nested_connect_doc(model, many));
                b.line(format!("connect?: {model_name}WhereUniqueInput"));
            }
        }, "}")
    }).to_string()
}

fn generate_model_create_or_connect_input(model: &Model, without: Option<&str>) -> String {
    let model_name = model.name();
    let without_title = if let Some(title) = without {
        let title = title.to_pascal_case();
        format!("Without{title}")
    } else {
        "".to_owned()
    };
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}ConnectOrCreate{without_title}Input = {{"), |b| {
            b.doc(unique_connect_doc(model));
            b.line(format!("where: {model_name}WhereUniqueInput"));
            b.doc(unique_connect_create_doc(model));
            b.line(format!("create: {model_name}Create{without_title}Input"));
        }, "}")
    }).to_string()
}

fn generate_model_create_input(graph: &Graph, model: &Model, without: Option<&str>, server_mode: bool) -> String {
    let model_name = model.name();
    let without_title = if let Some(title) = without {
        let title = title.to_pascal_case();
        format!("Without{title}")
    } else {
        "".to_owned()
    };
    let without_relation = if let Some(title) = without {
        Some(model.relation(title).unwrap())
    } else {
        None
    };
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}Create{without_title}Input = {{"), |b| {
            model.input_keys().iter().for_each(|k| {
                if let Some(field) = model.field(k) {
                    let field_name = &field.name;
                    let field_ts_type = field.field_type().to_typescript_create_input_type(field.optionality.is_optional(), server_mode);
                    if let Some(without_relation) = without_relation {
                        if !without_relation.fields().contains(k) {
                            b.doc(field_doc(field));
                            b.line(format!("{field_name}?: {field_ts_type}"));
                        }
                    } else {
                        b.doc(field_doc(field));
                        b.line(format!("{field_name}?: {field_ts_type}"));
                    }
                } else if let Some(relation) = model.relation(k) {
                    let relation_name = relation.name();
                    let relation_model_name = relation.model();
                    let relation_model = graph.model(relation_model_name).unwrap();
                    let num = if relation.is_vec() { "Many" } else { "One" };
                    if let Some(without_relation) = without_relation {
                        if without_relation.name() != k {
                            if let Some(opposite_relation) = relation_model.relations().iter().find(|r| {
                                r.fields() == relation.references() && r.references() == relation.fields()
                            }) {
                                let opposite_relation_name = opposite_relation.name().to_pascal_case();
                                b.doc(relation_doc(relation));
                                b.line(format!("{relation_name}?: {relation_model_name}CreateNested{num}Without{opposite_relation_name}Input"))
                            } else {
                                b.doc(relation_doc(relation));
                                b.line(format!("{relation_name}?: {relation_model_name}CreateNested{num}Input"))
                            }
                        }
                    } else {
                        if let Some(opposite_relation) = relation_model.relations().iter().find(|r| {
                            r.fields() == relation.references() && r.references() == relation.fields()
                        }) {
                            let opposite_relation_name = opposite_relation.name().to_pascal_case();
                            b.doc(relation_doc(relation));
                            b.line(format!("{relation_name}?: {relation_model_name}CreateNested{num}Without{opposite_relation_name}Input"))
                        } else {
                            b.doc(relation_doc(relation));
                            b.line(format!("{relation_name}?: {relation_model_name}CreateNested{num}Input"))
                        }
                    }
                }
            });
        }, "}");
    }).to_string()
}

fn generate_model_upsert_with_where_unique_input(model: &Model, without: Option<&str>) -> String {
    let model_name = model.name();
    let without_title = if let Some(title) = without {
        let title = title.to_pascal_case();
        format!("Without{title}")
    } else {
        "".to_owned()
    };
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}UpsertWithWhereUnique{without_title}Input = {{"), |b| {
            b.doc(unique_where_doc(model));
            b.line(format!("where: {model_name}WhereUniqueInput"));
            b.doc(create_or_update_doc(model, Action::from_u32(UPDATE_HANDLER)));
            b.line(format!("update: {model_name}Update{without_title}Input"));
            b.doc(create_or_update_doc(model, Action::from_u32(CREATE_HANDLER)));
            b.line(format!("create: {model_name}Create{without_title}Input"));
        }, "}")
    }).to_string()
}

fn generate_model_update_with_where_unique_input(model: &Model, without: Option<&str>) -> String {
    let model_name = model.name();
    let without_title = if let Some(title) = without {
        let title = title.to_pascal_case();
        format!("Without{title}")
    } else {
        "".to_owned()
    };
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}UpdateWithWhereUnique{without_title}Input = {{"), |b| {
            b.doc(unique_where_doc(model));
            b.line(format!("where: {model_name}WhereUniqueInput"));
            b.doc(create_or_update_doc(model, Action::from_u32(UPDATE_HANDLER)));
            b.line(format!("update: {model_name}Update{without_title}Input"));
        }, "}")
    }).to_string()
}

fn generate_model_update_many_with_where_input(model: &Model, without: Option<&str>) -> String {
    let model_name = model.name();
    let without_title = if let Some(title) = without {
        let title = title.to_pascal_case();
        format!("Without{title}")
    } else {
        "".to_owned()
    };
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}UpdateManyWithWhere{without_title}Input = {{"), |b| {
            b.doc(where_doc(model));
            b.line(format!("where: {model_name}WhereInput"));
            b.doc(create_or_update_doc(model, Action::from_u32(UPDATE_MANY_HANDLER)));
            b.line(format!("update: {model_name}Update{without_title}Input"));
        }, "}")
    }).to_string()
}

fn generate_model_update_nested_input(_graph: &Graph, model: &Model, without: Option<&str>, many: bool) -> String {
    let model_name = model.name();
    let without_title = if let Some(title) = without {
        let title = title.to_pascal_case();
        format!("Without{title}")
    } else {
        "".to_owned()
    };
    let many_title = if many { "Many" } else { "One" };
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}UpdateNested{many_title}{without_title}Input = {{"), |b| {
            if many {
                b.doc(nested_create_doc(model, many));
                b.line(format!("create?: Enumerable<{model_name}Create{without_title}Input>"));
                b.doc(nested_create_or_connect_doc(model, many));
                b.line(format!("connectOrCreate?: Enumerable<{model_name}ConnectOrCreate{without_title}Input>"));
                b.doc(nested_connect_doc(model, many));
                b.line(format!("connect?: Enumerable<{model_name}WhereUniqueInput>"));
                b.doc(nested_set_doc(model, many));
                b.line(format!("set?: Enumerable<{model_name}WhereUniqueInput>"));
                b.doc(nested_update_doc(model, many));
                b.line(format!("update?: Enumerable<{model_name}UpdateWithWhereUnique{without_title}Input>"));
                b.doc(nested_upsert_doc(model, many));
                b.line(format!("upsert?: Enumerable<{model_name}UpsertWithWhereUnique{without_title}Input>"));
                b.doc(nested_disconnect_doc(model, many));
                b.line(format!("disconnect?: Enumerable<{model_name}WhereUniqueInput>"));
                b.doc(nested_delete_doc(model, many));
                b.line(format!("delete?: Enumerable<{model_name}WhereUniqueInput>"));
                b.doc(nested_update_doc(model, true));
                b.line(format!("updateMany?: Enumerable<{model_name}UpdateManyWithWhere{without_title}Input>"));
                b.doc(nested_delete_doc(model, true));
                b.line(format!("deleteMany?: Enumerable<{model_name}WhereInput>"));
            } else {
                b.doc(nested_create_doc(model, many));
                b.line(format!("create?: {model_name}Create{without_title}Input"));
                b.doc(nested_create_or_connect_doc(model, many));
                b.line(format!("connectOrCreate?: {model_name}ConnectOrCreate{without_title}Input"));
                b.doc(nested_connect_doc(model, many));
                b.line(format!("connect?: {model_name}WhereUniqueInput"));
                b.doc(nested_set_doc(model, many));
                b.line(format!("set?: {model_name}WhereUniqueInput"));
                b.doc(nested_update_doc(model, many));
                b.line(format!("update?: {model_name}UpdateWithWhereUnique{without_title}Input"));
                b.doc(nested_upsert_doc(model, many));
                b.line(format!("upsert?: {model_name}UpsertWithWhereUnique{without_title}Input"));
                b.doc(nested_disconnect_doc(model, many));
                b.line(format!("disconnect?: boolean"));
                b.doc(nested_delete_doc(model, many));
                b.line(format!("delete?: boolean"));
            }
        }, "}")
    }).to_string()
}

fn generate_model_update_input(graph: &Graph, model: &Model, without: Option<&str>, server_mode: bool) -> String {
    let model_name = model.name();
    let without_title = if let Some(title) = without {
        let title = title.to_pascal_case();
        format!("Without{title}")
    } else {
        "".to_owned()
    };
    let without_relation = if let Some(title) = without {
        Some(model.relation(title).unwrap())
    } else {
        None
    };
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}Update{without_title}Input = {{"), |b| {
            model.input_keys().iter().for_each(|k| {
                if let Some(field) = model.field(k) {
                    let field_name = &field.name;
                    let field_ts_type = field.field_type().to_typescript_update_input_type(field.optionality.is_optional(), server_mode);
                    b.doc(field_doc(field));
                    b.line(format!("{field_name}?: {field_ts_type}"));
                } else if let Some(relation) = model.relation(k) {
                    let relation_name = relation.name();
                    let relation_model_name = relation.model();
                    let relation_model = graph.model(relation_model_name).unwrap();
                    let num = if relation.is_vec() { "Many" } else { "One" };
                    if let Some(without_relation) = without_relation {
                        if without_relation.name() != k {
                            if let Some(opposite_relation) = relation_model.relations().iter().find(|r| {
                                r.fields() == relation.references() && r.references() == relation.fields()
                            }) {
                                let opposite_relation_name = opposite_relation.name().to_pascal_case();
                                b.doc(relation_doc(relation));
                                b.line(format!("{relation_name}?: {relation_model_name}UpdateNested{num}Without{opposite_relation_name}Input"))
                            } else {
                                b.doc(relation_doc(relation));
                                b.line(format!("{relation_name}?: {relation_model_name}UpdateNested{num}Input"))
                            }
                        }
                    } else {
                        if let Some(opposite_relation) = relation_model.relations().iter().find(|r| {
                            r.fields() == relation.references() && r.references() == relation.fields()
                        }) {
                            let opposite_relation_name = opposite_relation.name().to_pascal_case();
                            b.doc(relation_doc(relation));
                            b.line(format!("{relation_name}?: {relation_model_name}UpdateNested{num}Without{opposite_relation_name}Input"))
                        } else {
                            b.doc(relation_doc(relation));
                            b.line(format!("{relation_name}?: {relation_model_name}UpdateNested{num}Input"))
                        }
                    }
                }
            });
        }, "}")
    }).to_string()
}

fn generate_model_scalar_update_input(_graph: &Graph, model: &Model) -> String {
    let model_name = model.name();
    Code::new(0, 4, |c| {
        c.block(format!("export type {model_name}ScalarUpdateInput = {{"), |b| {
            model.fields().iter().for_each(|field| {
                let field_name = &field.name;
                let field_ts_type = field.field_type().to_typescript_update_input_type(field.optionality.is_optional(), true);
                b.doc(field_doc(field));
                b.line(format!("{field_name}?: {field_ts_type}"));
            });
        }, "}")
    }).to_string()
}

fn generate_model_credentials_input(model: &Model) -> String {
    let model_name = model.name();
    Code::new(0, 4, |c| {
        c.block(format!(r#"export type {model_name}CredentialsInput = {{"#), |b| {
            let auth_identity_keys = model.auth_identity_keys();
            let auth_by_keys = model.auth_by_keys();
            let auth_identity_optional = auth_identity_keys.len() != 1;
            let auth_by_keys_optional = auth_by_keys.len() != 1;
            for key in auth_identity_keys {
                let field = model.field(key).unwrap();
                let field_name = &field.name;
                let field_type = field.field_type().to_typescript_type(auth_identity_optional);
                b.doc(field_doc(field));
                b.line(format!("{field_name}: {field_type}"));
            }
            for key in auth_by_keys {
                let field = model.field(key).unwrap();
                let field_name = &field.name;
                let field_type = field.field_type().to_typescript_type(auth_by_keys_optional);
                b.doc(field_doc(field));
                b.line(format!("{field_name}: {field_type}"));
            }
        }, "}");
    }).to_string()
}

pub(crate) fn generate_index_d_ts(graph: &Graph, client_obj_name: String, server_mode: bool) -> String {
    let decimal = if !server_mode {
        "./decimal"
    } else {
        "decimal.js"
    };
    Code::new(0, 4, |c| {
        c.line(format!(r#"import Decimal from "{decimal}""#));
        c.line(if server_mode {
            generate_server_runtime_types()
        } else {
            generate_client_runtime_types()
        });
        c.line(generate_filter_types(server_mode));
        c.line(generate_operation_types(server_mode));
        if !server_mode {
            c.line(r#"
export * from "./decimal"

export declare function setBearerToken(token: string | undefined)

export declare function getBearerToken(): string | undefined

export declare class TeoError extends Error {

    type: string
    errors: {[key: string]: string} | null

    constructor(responseError: ResponseError)

    get name(): string
}"#);
        }
        c.empty_line();
        // enum definitions
        graph.enums().iter().for_each(|e| {
            let name = e.0;
            let choices = e.1.values().iter().map(|i| {String::from("\"") + i + "\""}).collect::<Vec<String>>().join(" | ");
            c.line(format!("export type {name} = {choices}"));
            c.empty_line();
        });
        if !server_mode {
            // model definitions
            graph.models().iter().for_each(|m| {
                if !m.is_teo_internal() {
                    let model_name = m.name();
                    c.block(format!("export type {model_name} = {{"), |b| {
                        m.output_keys().iter().for_each(|k| {
                            if let Some(field) = m.field(k) {
                                let field_name = &field.name;
                                let field_type = field.field_type().to_typescript_type(field.optionality.is_optional());
                                b.line(format!("{field_name}: {field_type}"));
                            }
                        });
                    }, "}");
                    c.empty_line();
                }
            });
        }
        // model input arguments
        graph.models().iter().for_each(|m| {
            if m.is_teo_internal() {
                return
            }
            let model_name = m.name();
            // select
            c.block(format!("export type {model_name}Select = {{"), |b| {
                m.output_keys().iter().for_each(|k| {
                    if let Some(field) = m.field(k) {
                        let field_name = &field.name;
                        b.doc(field_doc(field));
                        b.line(format!("{field_name}?: boolean"));
                    }
                })
            }, "}");
            // include
            c.block(format!("export type {model_name}Include = {{"), |b| {
                for relation in m.relations() {
                    let name = relation.name();
                    let is_vec = relation.is_vec();
                    let find_many = if is_vec { "FindMany" } else { "" };
                    let r_model = relation.model();
                    b.doc(relation_doc(relation));
                    b.line(format!("{name}?: boolean | {r_model}{find_many}Args"));
                }
            }, "}");
            // where
            c.block(format!("export type {model_name}WhereInput = {{"), |b| {
                for op in ["AND", "OR", "NOT"] {
                    b.line(format!("{op}?: Enumerable<{model_name}WhereInput>"));
                }
                m.query_keys().iter().for_each(|k| {
                    if let Some(field) = m.field(k) {
                        let field_name = &field.name;
                        let field_filter = field.field_type().to_typescript_filter_type(field.optionality.is_optional(), server_mode);
                        b.doc(field_doc(field));
                        b.line(format!("{field_name}?: {field_filter}"));
                    } else if let Some(relation) = m.relation(k) {
                        let list = if relation.is_vec() { "List" } else { "" };
                        let relation_name = relation.name();
                        let relation_model = relation.model();
                        b.doc(relation_doc(relation));
                        b.line(format!("{relation_name}?: {relation_model}{list}RelationFilter"));
                    }
                })
            }, "}");
            // where unique
            c.block(format!("export type {model_name}WhereUniqueInput = {{"), |b| {
                let mut used_field_names: Vec<&str> = Vec::new();
                m.indices().iter().for_each(|index| {
                    if index.r#type() == Primary || index.r#type() == Unique {
                        index.items().iter().for_each(|item| {
                            if !used_field_names.contains(&&***&&item.field_name()) {
                                if let Some(field) = m.field(&item.field_name()) {
                                    let ts_type = field.field_type().to_typescript_type(field.optionality.is_optional());
                                    let field_name = &item.field_name();
                                    b.doc(field_doc(field));
                                    b.line(format!("{field_name}?: {ts_type}"));
                                }
                                used_field_names.push(item.field_name());
                            }
                        });
                    }
                });
            }, "}");
            // relation filter
            c.block(format!("export type {model_name}RelationFilter = {{"), |b| {
                b.line(format!("is?: {model_name}WhereInput"));
                b.line(format!("isNot?: {model_name}WhereInput"));
            }, "}");
            // list relation filter
            c.block(format!("export type {model_name}ListRelationFilter = {{"), |b| {
                b.line(format!("every?: {model_name}WhereInput"));
                b.line(format!("some?: {model_name}WhereInput"));
                b.line(format!("none?: {model_name}WhereInput"));
            }, "}");
            // order by
            c.block(format!("export type {model_name}OrderByInput = {{"), |b| {
                m.query_keys().iter().for_each(|k| {
                    if let Some(field) = m.field(k) {
                        let field_name = &field.name;
                        b.doc(field_doc(field));
                        b.line(format!("{field_name}?: SortOrder"));
                    } else if let Some(relation) = m.relation(k) {
                        let _relation_model = relation.model();
                        let _relation_name = relation.name();
                        //b.line(format!("{relation_name}?: {relation_model}OrderByRelationAggregateInput"));
                    }
                })
            }, "}");
            // create and update inputs without anything
            c.line(generate_model_create_input(graph, m, None, server_mode));
            c.line(generate_model_create_nested_input(graph, m, None, true));
            c.line(generate_model_create_nested_input(graph, m, None, false));
            c.line(generate_model_create_or_connect_input(m, None));
            m.relations().iter().for_each(|r| {
                c.line(generate_model_create_input(graph, m, Some(r.name()), server_mode));
                c.line(generate_model_create_nested_input(graph, m, Some(r.name()), true));
                c.line(generate_model_create_nested_input(graph, m, Some(r.name()), false));
                c.line(generate_model_create_or_connect_input(m, Some(r.name())));
            });
            c.line(generate_model_update_input(graph, m, None, server_mode));
            c.line(generate_model_update_nested_input(graph, m, None, true));
            c.line(generate_model_update_nested_input(graph, m, None, false));
            c.line(generate_model_upsert_with_where_unique_input(m, None));
            c.line(generate_model_update_with_where_unique_input(m, None));
            c.line(generate_model_update_many_with_where_input(m, None));
            m.relations().iter().for_each(|r| {
                c.line(generate_model_update_input(graph, m, Some(r.name()), server_mode));
                c.line(generate_model_update_nested_input(graph, m, Some(r.name()), true));
                c.line(generate_model_update_nested_input(graph, m, Some(r.name()), false));
                c.line(generate_model_upsert_with_where_unique_input(m, Some(r.name())));
                c.line(generate_model_update_with_where_unique_input(m, Some(r.name())));
                c.line(generate_model_update_many_with_where_input(m, Some(r.name())));
            });
            if m.identity() {
                c.line(generate_model_credentials_input(m));
            }
            if server_mode {
                c.line(generate_model_scalar_update_input(graph, m));
            }
            // action args
            c.block(format!(r#"export type {model_name}Args = {{"#), |b| {
                b.doc(select_doc(m));
                b.line(format!(r#"select?: {model_name}Select"#));
                b.doc(include_doc(m));
                b.line(format!(r#"include?: {model_name}Include"#));
            }, "}");
            Action::handlers_iter().for_each(|a| {
                if !m.has_action(*a) { return }
                let action_name = a.as_handler_str();
                let capitalized_action_name = action_name.to_pascal_case();
                c.block(format!(r#"export type {model_name}{capitalized_action_name}Args = {{"#), |b| {
                    if a.handler_requires_where() {
                        if a == &Action::from_u32(FIND_FIRST_HANDLER) {
                            b.doc(where_doc_first(m));
                        } else {
                            b.doc(where_doc(m));
                        }
                        b.line(format!(r#"where?: {model_name}WhereInput"#));
                    }
                    if a.handler_requires_where_unique() {
                        b.doc(unique_where_doc(m));
                        b.line(format!(r#"where?: {model_name}WhereUniqueInput"#));
                    }
                    b.doc(select_doc(m));
                    b.line(format!(r#"select?: {model_name}Select"#));
                    b.doc(include_doc(m));
                    b.line(format!(r#"include?: {model_name}Include"#));
                    if a.handler_requires_where() {
                        b.doc(order_by_doc(m));
                        b.line(format!(r#"orderBy?: Enumerable<{model_name}OrderByInput>"#));
                        b.doc(cursor_doc(m));
                        b.line(format!(r#"cursor?: {model_name}WhereUniqueInput"#));
                        b.doc(take_doc(m));
                        b.line(format!(r#"take?: number"#));
                        b.doc(skip_doc(m));
                        b.line(format!(r#"skip?: number"#));
                        b.doc(page_size_doc(m));
                        b.line(format!(r#"pageSize?: number"#));
                        b.doc(page_number_doc(m));
                        b.line(format!(r#"pageNumber?: number"#));
                        //b.line(format!{r#"distinct? {model_name}ScalarFieldEnum"#})
                    }
                    if a.handler_requires_create() {
                        b.doc(create_or_update_doc(m, if a == &Action::from_u32(UPSERT_HANDLER) { Action::from_u32(CREATE_HANDLER) } else { a.clone() }));
                        b.line(format!(r#"create: {model_name}CreateInput"#));
                    }
                    if a.handler_requires_update() {
                        b.doc(create_or_update_doc(m, if a == &Action::from_u32(UPSERT_HANDLER) { Action::from_u32(UPDATE_HANDLER) } else { a.clone() }));
                        b.line(format!(r#"update: {model_name}UpdateInput"#));
                    }
                    if a.handler_requires_credentials() {
                        b.doc(credentials_doc(m, *a));
                        b.line(format!(r#"credentials: {model_name}CredentialsInput"#))
                    }
                }, "}");
            });
            // get payload is for typescript only
            c.block(format!("export type {model_name}GetPayload<S extends boolean | null | undefined | {model_name}Args, U = keyof S> = S extends true"), |b| {
                b.line(format!("? {model_name}"));
                b.block(": S extends undefined", |b| {
                    b.line("? never");
                    b.block(format!(": S extends {model_name}Args | {model_name}FindManyArgs"), |b| {
                        b.block("? 'include' extends U", |b| {
                            b.block(format!("? SelectSubset<{model_name}, S> & {{"), |b| {
                                b.block(format!("[P in ExistKeys<S['include']>]:"), |b| {
                                    for relation in m.relations() {
                                        let name = relation.name();
                                        let is_array = relation.is_vec();
                                        let required = relation.is_required();
                                        let required_mark = if required { "" } else { " | undefined" };
                                        let r_model = relation.model();
                                        let array_prefix = if is_array { "Array<" } else { "" };
                                        let array_suffix = if is_array { ">" } else { "" };
                                        b.line(format!("P extends '{name}' ? {array_prefix}{r_model}GetPayload<S['include'][P]>{array_suffix}{required_mark} :"));
                                    }
                                }, "never");
                            }, "}");
                            b.line(format!(": SelectSubset<{model_name}, S>"));
                        }, format!(": {model_name}"));
                    }, "");
                }, "");
            }, "")
        });
        if !server_mode {
            // delegates
            let object_name = client_obj_name.clone();
            let object_class_name = object_name.to_pascal_case();
            graph.models().iter().for_each(|m| {
                if m.is_teo_internal() {
                    return
                }
                if m.actions().len() > 0 {
                    let model_name = m.name();
                    let model_var_name = model_name.to_camel_case();
                    let model_class_name = model_var_name.to_pascal_case();
                    c.block(format!("declare class {model_class_name}Delegate {{"), |b| {
                        Action::handlers_iter().for_each(|a| {
                            if m.has_action(*a) {
                                let action_var_name = a.as_handler_str().to_camel_case();
                                let action_capitalized_name = action_var_name.to_pascal_case();
                                let res_meta = match a.handler_res_meta() {
                                    ResMeta::PagingInfo => "PagingInfo",
                                    ResMeta::TokenInfo => "TokenInfo",
                                    ResMeta::NoMeta => "undefined",
                                    ResMeta::Other => "undefined",
                                };
                                let res_data = match a.handler_res_data() {
                                    ResData::Single => model_name.to_string(),
                                    ResData::Vec => model_name.to_string() + "[]",
                                    ResData::Other => "never".to_string(),
                                    ResData::Number => "number".to_string(),
                                };
                                let payload_array = match a.handler_res_data() {
                                    ResData::Vec => "[]",
                                    _ => ""
                                };
                                b.empty_line();
                                b.doc(action_doc(&object_name, a.clone(), m));
                                b.line(format!("{action_var_name}<T extends {model_name}{action_capitalized_name}Args>(args?: T): Promise<Response<{res_meta}, CheckSelectInclude<T, {res_data}, {model_name}GetPayload<T>{payload_array}>>>"));
                            }
                        });
                    }, "}");
                    c.empty_line();
                }
            });
            // main interface
            c.block(format!("declare class {object_class_name} {{"), |b| {
                graph.models().iter().for_each(|m| {
                    if m.is_teo_internal() {
                        return
                    }
                    if m.actions().len() > 0 {
                        let model_name = m.name();
                        let model_var_name = model_name.to_camel_case();
                        let model_class_name = model_var_name.to_pascal_case();
                        b.doc(action_group_doc(&object_name, m));
                        b.line(format!("{model_var_name}: {model_class_name}Delegate"));
                    }
                });
                b.line("constructor(token?: string)");
                b.doc(with_token_doc());
                b.line(format!("$withToken(token?: string): {}", object_class_name));
            }, "}");
            c.empty_line();
            c.line(main_object_doc(&object_name, graph));
            c.line(format!("export const {object_name}: {object_class_name}"));
        }
    }).to_string()
}
