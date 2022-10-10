use key_path::KeyPath;
use crate::connectors::sql::schema::dialect::SQLDialect;
use crate::connectors::sql::stmts::select::r#where::{ToWrappedSQLString, WhereClause};
use crate::connectors::sql::stmts::select::r#where::WhereClause::And;
use crate::core::error::ActionError;
use crate::core::field::r#type::FieldType;
use crate::core::input::Input;
use crate::core::model::Model;
use crate::core::result::ActionResult;
use crate::prelude::{Graph, Object, Value};

pub(crate) struct Query { }

impl Query {

    fn where_item(col_name: &str, op: &str, val: &str) -> String {
        format!("{col_name} {op} {val}")
    }

    pub(crate) fn where_from_identifier(object: &Object, dialect: SQLDialect) -> String {
        Self::where_from_value(object.model(), object.graph(), &object.identifier(), dialect)
    }

    fn where_entry_array(
        column_name: &str,
        r#type: &FieldType,
        optional: bool,
        value: &Value,
        graph: &Graph,
        op: &str
    ) -> String {
        let arr_val = value.as_vec().unwrap();
        let mut arr: Vec<String> = Vec::new();
        for val in arr_val {
            arr.push(val.to_sql_string(r#type, optional, graph));
        }
        sql_where_item(column_name, op, arr.join(", ").to_wrapped())
    }

    fn where_entry_item(
        column_name: &str,
        r#type: &FieldType,
        optional: bool,
        value: &Value,
        graph: &Graph,
        dialect: SQLDialect,
    ) -> String {
        if let Some(map) = value.as_hashmap() {
            let mut result: Vec<String> = vec![];
            for (key, value) in map {
                match key.as_str() {
                    "equals" => {
                        result.push(Self::sql_where_item(column_name, "=", value.to_sql_string(r#type, optional, graph)));
                    }
                    "not" => {
                        result.push(Self::sql_where_item(column_name, "<>", value.to_sql_string(r#type, optional, graph)));
                    }
                    "gt" => {
                        result.push(Self::sql_where_item(column_name, ">", value.to_sql_string(r#type, false, graph)));
                    }
                    "gte" => {
                        result.push(Self::sql_where_item(column_name, ">=", value.to_sql_string(r#type, false, graph)));
                    }
                    "lt" => {
                        result.push(Self::sql_where_item(column_name, "<", value.to_sql_string(r#type, false, graph)));
                    }
                    "lte" => {
                        result.push(Self::sql_where_item(column_name, "<=", value.to_sql_string(r#type, false, graph)));
                    }
                    "in" => {
                        result.push(Self::where_entry_array(column_name, r#type, optional, value, graph, "IN")?);
                    }
                    "notIn" => {
                        result.push(Self::where_entry_array(column_name, r#type, optional, value, graph, "NOT IN")?);
                    }
                    "contains" => {
                        let i_mode = Input::has_i_mode(map);
                        result.push(Self::sql_where_item(&column_name.to_i_mode(i_mode), "LIKE", value.to_sql_string(r#type, false, graph).to_like(true, true).to_i_mode(i_mode)));
                    }
                    "startsWith" => {
                        let i_mode = Input::has_i_mode(map);
                        result.push(Self::sql_where_item(&column_name.to_i_mode(i_mode), "LIKE", value.to_sql_string(r#type, false, graph).to_like(false, true).to_i_mode(i_mode)));
                    }
                    "endsWith" => {
                        let i_mode = Input::has_i_mode(map);
                        result.push(Self::sql_where_item(&column_name.to_i_mode(i_mode), "LIKE", value.to_sql_string(r#type, false, graph).to_like(true, false).to_i_mode(i_mode)));
                    }
                    "matches" => {
                        let i_mode = Input::has_i_mode(map);
                        result.push(Self::sql_where_item(&column_name.to_i_mode(i_mode), "REGEXP", value.to_sql_string(r#type, false, graph).to_i_mode(i_mode)));
                    }
                    "mode" => { }
                    "has" => {
                        let element_type = r#type.element_field().unwrap();
                        result.push(Self::sql_where_item(column_name, "@>", value.to_sql_string(element_type.r#type(), element_type.is_optional(), graph).wrap_in_array()));
                    }
                    "hasEvery" => {
                        result.push(Self::sql_where_item(column_name, "@>", value.to_sql_string(r#type, false, graph)));
                    }
                    "hasSome" => {
                        result.push(Self::sql_where_item(column_name, "&&", value.to_sql_string(r#type, false, graph)));
                    }
                    "isEmpty" => {
                        result.push(Self::sql_where_item(&format!("ARRAY_LENGTH({})", column_name), "=", "0".to_owned()));
                    }
                    "length" => {
                        result.push(Self::sql_where_item(&format!("ARRAY_LENGTH({})", column_name), "=", value.to_sql_string(&FieldType::U64, false, graph)));
                    }
                    _ => panic!("Unhandled key."),
                }
            }
            And(result).to_wrapped_string(dialect)
        } else {
            sql_where_item(column_name, "=", value.to_sql_string(r#type, optional, graph))
        }
    }

    fn where_entry(
        column_name: &str,
        field_type: &FieldType,
        optional: bool,
        value: &Value,
        graph: &Graph,
        dialect: SQLDialect,
    ) -> String {
        Self::where_entry_item(column_name, field_type, optional, value, graph, dialect)
    }

    pub(crate) fn where_from_value(model: &Model, _graph: &Graph, identifier: &Value, dialect: SQLDialect) -> String {
        let mut retval: Vec<String> = vec![];
        for (key, value) in identifier.as_hashmap().unwrap() {
            let field = model.field(key).unwrap();
            let column_name = field.column_name();
            retval.push(format!("{} = {}", column_name, value.to_string(dialect)));
        }
        And(retval).to_string(dialect)
    }

    pub(crate) fn r#where(model: &Model, graph: &Graph, r#where: &Value, dialect: SQLDialect) -> String {
        let r#where = r#where.as_hashmap().unwrap();
        let mut retval: Vec<String> = vec![];
        for (key, value) in r#where.iter() {
            if key == "AND" {
                let path = key_path.as_ref() + "AND";
                let inner = WhereClause::And(value.as_vec().unwrap().iter().map(|w| Self::r#where(model, graph, w, dialect).unwrap().unwrap()).collect()).to_string(dialect);
                let val = "(".to_owned() + &inner + ")";
                retval.push(val);
            } else if key == "OR" {
                let path = key_path.as_ref() + "OR";
                let inner = WhereClause::Or(value.as_vec().unwrap().iter().map(|w| Self::r#where(model, graph, value, dalect).unwrap().unwrap()).collect()).to_string(dialect);
                let val = "(".to_owned() + &inner + ")";
                retval.push(val);
            } else if key == "NOT" {
                let path = key_path.as_ref() + "NOT";
                let inner = WhereClause::Not(Self::r#where(model, graph, value, dialect).unwrap().unwrap()).to_string(dialect);
                let val = "(".to_owned() + &inner + ")";
                retval.push(val);
            } else {
                if let Some(field) = model.field(key) {
                    let column_name = field.column_name();
                    let optional = field.optionality.is_optional();
                    let where_entry = parse_sql_where_entry(column_name, &field.field_type, optional, value, graph, dialect, key_path.as_ref())?;
                    retval.push(where_entry);
                } else if let Some(relation) = model.relation(key) {
                    panic!("not handle this yet")
                }
            }
        }
        And(retval).to_string(dialect)
    }

    pub(crate) fn order_by(
        model: &Model,
        graph: &Graph,
        order_by: &Value,
        dialect: SQLDialect,
    ) -> String {
        let order_by = order_by.as_hashmap().unwrap();
        let mut retval: Vec<String> = vec![];
        for (key, value) in order_by.iter() {
            if let Some(field) = model.field(key) {
                let column_name = field.column_name();
                if let Some(str) = value.as_str() {
                    match str {
                        "asc" => retval.push(format!("{} ASC", column_name)),
                        "desc" => retval.push(format!("{} DESC", column_name)),
                        _ => panic!("Unhandled."),
                    }
                }
            }
        }
        retval.join(",")
    }
}