pub(crate) mod resolver;
use snailquote::unescape;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::fs;
use std::sync::{Arc, Mutex};
use maplit::btreemap;
use pest::Parser as PestParser;
use crate::parser::ast::argument::{Argument, ArgumentList};
use crate::parser::ast::client::Client;
use crate::parser::ast::config::Config;
use crate::parser::ast::connector::Connector;
use crate::parser::ast::constant::Constant;
use crate::parser::ast::decorator::Decorator;
use crate::parser::ast::expression::{Expression, ExpressionKind, ArrayLiteral, BoolLiteral, DictionaryLiteral, EnumChoiceLiteral, NullLiteral, NumericLiteral, RangeLiteral, StringLiteral, TupleLiteral, RegExpLiteral, NullishCoalescing};
use crate::parser::ast::field::Field;
use crate::parser::ast::generator::Generator;
use crate::parser::ast::group::Group;
use crate::parser::ast::identifier::Identifier;
use crate::parser::ast::import::Import;
use crate::parser::ast::item::Item;
use crate::parser::ast::model::Model;
use crate::parser::ast::pipeline::Pipeline;
use crate::parser::ast::r#enum::{Enum, EnumChoice};
use crate::parser::ast::r#type::{Arity, Type};
use crate::parser::ast::source::Source;
use crate::parser::ast::span::Span;
use crate::parser::ast::subscript::Subscript;
use crate::parser::ast::top::Top;
use crate::parser::ast::unit::Unit;
use crate::parser::parser::resolver::Resolver;

#[derive(pest_derive::Parser)]
#[grammar = "./src/parser/schema.pest"]
pub(crate) struct SchemaParser;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

#[derive(Debug)]
pub(crate) struct Parser {
    pub(crate) main: Option<Arc<Mutex<Source>>>,
    pub(crate) sources: HashMap<usize, Arc<Mutex<Source>>>,
    pub(crate) tops: HashMap<usize, Arc<Mutex<Top>>>,
    pub(crate) connector: Option<Arc<Mutex<Top>>>,
    pub(crate) next_id: usize,
}

impl Parser {

    pub(crate) fn new() -> Self {
        Self {
            main: None,
            sources: HashMap::new(),
            tops: HashMap::new(),
            connector: None,
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    pub(crate) fn parse(&mut self, main: Option<&str>) -> () {
        let main = main.unwrap_or("schema.teo");
        let relative = PathBuf::from(main);
        let absolute = match fs::canonicalize(&relative) {
            Ok(path) => path,
            Err(_) => panic!("Schema file '{}' is not found.", relative.to_str().unwrap()),
        };
        let id = self.next_id();
        let source = self.parse_source(absolute, id);
        self.main = Some(source.clone());
        // println!("{:?}", self);
        Resolver::resolve_parser(self);
    }

    fn parse_source(&mut self, path: PathBuf, id: usize) -> Arc<Mutex<Source>> {
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => panic!("{}", err)
        };
        let mut pairs = match SchemaParser::parse(Rule::schema, &content) {
            Ok(pairs) => pairs,
            Err(err) => panic!("{}", err)
        };
        let pairs = pairs.next().unwrap();
        let mut tops: Vec<Arc<Mutex<Top>>> = vec![];
        let mut imports: BTreeMap<usize, Arc<Mutex<Top>>> = btreemap!{};
        let mut constants: BTreeMap<usize, Arc<Mutex<Top>>> = btreemap!{};
        let mut models: BTreeMap<usize, Arc<Mutex<Top>>> = btreemap!{};
        let mut pairs = pairs.into_inner().peekable();
        while let Some(current) = pairs.next() {
            let item_id = self.next_id();
            match current.as_rule() {
                Rule::import_statement => {
                    let import = self.parse_import(current, id, item_id);
                    tops.push(import.clone());
                    imports.insert(item_id, import.clone());
                },
                Rule::let_declaration => {
                    let constant = self.parse_let_declaration(current, id, item_id);
                    tops.push(constant.clone());
                    imports.insert(item_id, constant.clone());
                },
                Rule::model_declaration => {
                    let model = self.parse_model(current, id, item_id);
                    tops.push(model.clone());
                    models.insert(item_id, model);
                },
                Rule::enum_declaration => tops.push(self.parse_enum(current, id)),
                Rule::config_declaration => tops.push(self.parse_config_block(current, id)),
                Rule::EOI | Rule::EMPTY_LINES => {},
                Rule::CATCH_ALL => panic!("Found catch all."),
                _ => panic!("Parsing panic!"),
            }
        }
        let result = Arc::new(Mutex::new(Source {
            id,
            path: path.clone(),
            tops, imports: imports.clone(),
            constants: constants.clone(),
            models: models.clone(),
        }));
        self.sources.insert(id, result.clone());
        for import in imports {
            let import = import.lock().unwrap();
            let import = import.as_import().unwrap();
            let relative = unescape(&import.source.value).unwrap();
            let relative = PathBuf::from(relative);
            let mut dir = path.clone();
            dir.pop();
            let new = dir.join(&relative);
            let absolute = match fs::canonicalize(&new) {
                Ok(path) => path,
                Err(_) => panic!("Schema file '{}' is not found.", relative.to_str().unwrap()),
            };
            let found = self.sources.values().find(|v| {
                v.lock().unwrap().path == absolute
            });
            if found.is_none() {
                let id = self.next_id();
                self.parse_source(absolute, id);
            }
        }
        result
    }

    fn parse_import(&mut self, pair: Pair<'_>, source_id: usize, item_id: usize) -> Arc<Mutex<Top>> {
        let mut identifiers = vec![];
        let span = Self::parse_span(&pair);
        let mut source: Option<StringLiteral> = None;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::string_literal => source = Some(StringLiteral { value: current.as_str().to_string(), span }),
                Rule::import_identifier_list => identifiers = Self::parse_import_identifier_list(current),
                _ => panic!("error."),
            }
        }
        Arc::new(Mutex::new(Top::Import(Import { id: item_id, source_id, identifiers, source: source.unwrap(), span })))
    }

    fn parse_model(&mut self, pair: Pair<'_>, source_id: usize, item_id: usize) -> Arc<Mutex<Top>> {
        let mut identifier: Option<Identifier> = None;
        let mut fields: Vec<Field> = vec![];
        let mut decorators: Vec<Decorator> = vec![];
        let span = Self::parse_span(&pair);
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::MODEL_KEYWORD | Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => {}
                Rule::identifier => identifier = Some(Self::parse_identifier(&current)),
                Rule::field_declaration => fields.push(Self::parse_field(current)),
                _ => panic!("error."),
            }
        }
        let result = Arc::new(Mutex::new(Top::Model(Model::new(
            item_id,
            source_id,
            identifier.unwrap(),
            fields,
            decorators,
            span,
        ))));
        self.tops.insert(item_id, result.clone());
        result
    }

    fn parse_field(pair: Pair<'_>) -> Field {
        let mut identifier: Option<Identifier> = None;
        let mut r#type: Option<Type> = None;
        let mut decorators: Vec<Decorator> = vec![];
        let span = Self::parse_span(&pair);
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::COLON => {},
                Rule::identifier => identifier = Some(Self::parse_identifier(&current)),
                Rule::field_type => r#type = Some(Self::parse_type(current)),
                Rule::item_decorator => decorators.push(Self::parse_decorator(current)),
                _ => panic!("error."),
            }
        }
        Field::new(
            identifier.unwrap(),
            r#type.unwrap(),
            decorators,
            span,
        )
    }

    fn parse_enum(&mut self, pair: Pair<'_>, source_id: usize) -> Arc<Mutex<Top>> {
        let mut identifier: Option<Identifier> = None;
        let mut choices: Vec<EnumChoice> = vec![];
        let result = Arc::new(Mutex::new(Top::Enum(Enum {
            id: self.next_id(),
            source_id,
            identifier: identifier.unwrap(),
            choices,
            span: Self::parse_span(&pair),
        })));
        self.tops.insert(result.lock().unwrap().id(), result.clone());
        result
    }

    fn parse_let_declaration(&mut self, pair: Pair<'_>, source_id: usize, item_id: usize) -> Arc<Mutex<Top>> {
        let span = Self::parse_span(&pair);
        let mut identifier: Option<Identifier> = None;
        let mut expression: Option<Expression> = None;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::identifier => identifier = Some(Self::parse_identifier(&current)),
                Rule::expression => expression = Some(Self::parse_expression(current)),
                _ => panic!("error."),
            }
        }
        Arc::new(Mutex::new(Top::Constant(Constant { id: item_id, source_id, identifier: identifier.unwrap(), expression: expression.unwrap(), span })))
    }

    fn parse_config_block(&mut self, pair: Pair<'_>, source_id: usize) -> Arc<Mutex<Top>> {
        let mut identifier: Option<Identifier> = None;
        let mut items: Vec<Item> = vec![];
        let mut keyword = "";
        let span = Self::parse_span(&pair);
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::BLOCK_OPEN | Rule::BLOCK_CLOSE => (),
                Rule::config_keywords => keyword = current.as_str(),
                Rule::identifier => identifier = Some(Self::parse_identifier(&current)),
                Rule::config_item => items.push(Self::parse_config_item(current)),
                _ => panic!("error."),
            }
        }
        let result = Arc::new(Mutex::new(match keyword {
            "config" => Top::Config(Config { items, span, source_id }),
            "connector" => Top::Connector(Connector::new(items, span, source_id)),
            "generator" => Top::Generator(Generator { identifier: identifier.unwrap(), items, span, source_id }),
            "client" => Top::Client(Client { identifier: identifier.unwrap(), items, span, source_id }),
            _ => panic!(),
        }));
        self.tops.insert(result.lock().unwrap().id(), result.clone());
        if keyword == "connector" {
            self.connector = Some(result.clone());
        }
        result
    }

    fn parse_config_item(pair: Pair<'_>) -> Item {
        let span = Self::parse_span(&pair);
        let mut identifier: Option<Identifier> = None;
        let mut expression: Option<Expression> = None;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::identifier => identifier = Some(Self::parse_identifier(&current)),
                Rule::expression => expression = Some(Self::parse_expression(current)),
                _ => panic!("error."),
            }
        }
        Item { identifier: identifier.unwrap(), expression: expression.unwrap(), span }
    }

    fn parse_decorator(pair: Pair<'_>) -> Decorator {
        let span = Self::parse_span(&pair);
        let mut unit: Option<ExpressionKind> = None;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::identifier_unit => unit = Some(Self::parse_unit(current)),
                _ => panic!(),
            }
        }
        Decorator {
            expression: unit.unwrap(),
            span,
        }
    }

    fn parse_pipeline(pair: Pair<'_>) -> Pipeline {
        let span = Self::parse_span(&pair);
        let mut unit: Option<ExpressionKind> = None;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::identifier_unit => unit = Some(Self::parse_unit(current)),
                _ => panic!(),
            }
        }
        Pipeline {
            expression: Box::new(unit.unwrap()),
            span,
        }
    }

    fn parse_argument(pair: Pair<'_>) -> Argument {
        let span = Self::parse_span(&pair);
        let mut name: Option<Identifier> = None;
        let mut value: Option<ExpressionKind> = None;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::named_argument => {
                    return Self::parse_named_argument(current);
                },
                Rule::expression => value = Some(Self::parse_expression(current).kind),
                Rule::empty_argument => panic!("Empty argument found."),
                _ => panic!(),
            }
        }
        Argument { name, value: value.unwrap(), span, resolved: None }
    }

    fn parse_named_argument(pair: Pair<'_>) -> Argument {
        let span = Self::parse_span(&pair);
        let mut name: Option<Identifier> = None;
        let mut value: Option<ExpressionKind> = None;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::identifier => name = Some(Self::parse_identifier(&current)),
                Rule::expression => value = Some(Self::parse_expression(current).kind),
                Rule::empty_argument => panic!("Empty argument found."),
                _ => panic!(),
            }
        }
        Argument { name, value: value.unwrap(), span, resolved: None }
    }

    fn parse_expression(pair: Pair<'_>) -> Expression {
        let span = Self::parse_span(&pair);
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::nullish_coalescing => return Expression::new(ExpressionKind::NullishCoalescing(Self::parse_nullish_coalescing(current))),
                Rule::unit => return Expression::new(Self::parse_unit(current)),
                Rule::pipeline => return Expression::new(ExpressionKind::Pipeline(Self::parse_pipeline(current))),
                _ => panic!(),
            }
        }
        panic!();
    }

    fn parse_unit(pair: Pair<'_>) -> ExpressionKind {
        let span = Self::parse_span(&pair);
        let mut unit = Unit { expressions: vec![], span };
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::group => unit.expressions.push(ExpressionKind::Group(Self::parse_group(current))),
                Rule::null_literal => unit.expressions.push(ExpressionKind::NullLiteral(NullLiteral { value: current.as_str().to_string(), span })),
                Rule::bool_literal => unit.expressions.push(ExpressionKind::BoolLiteral(BoolLiteral { value: current.as_str().to_string(), span })),
                Rule::numeric_literal => unit.expressions.push(ExpressionKind::NumericLiteral(NumericLiteral { value: current.as_str().to_string(), span })),
                Rule::string_literal => unit.expressions.push(ExpressionKind::StringLiteral(StringLiteral { value: current.as_str().to_string(), span })),
                Rule::regexp_literal => unit.expressions.push(ExpressionKind::RegExpLiteral(RegExpLiteral { value: current.as_str().to_string(), span })),
                Rule::enum_choice_literal => unit.expressions.push(ExpressionKind::EnumChoiceLiteral(EnumChoiceLiteral { value: current.as_str().to_string(), span })),
                Rule::tuple_literal => unit.expressions.push(ExpressionKind::TupleLiteral(Self::parse_tuple_literal(current))),
                Rule::array_literal => unit.expressions.push(ExpressionKind::ArrayLiteral(Self::parse_array_literal(current))),
                Rule::dictionary_literal => unit.expressions.push(ExpressionKind::DictionaryLiteral(Self::parse_dictionary_literal(current))),
                Rule::range_literal => unit.expressions.push(ExpressionKind::RangeLiteral(Self::parse_range_literal(current))),
                Rule::identifier => unit.expressions.push(ExpressionKind::Identifier(Self::parse_identifier(&current))),
                Rule::subscript => unit.expressions.push(ExpressionKind::Subscript(Self::parse_subscript(current))),
                Rule::argument_list => unit.expressions.push(ExpressionKind::ArgumentList(Self::parse_argument_list(current))),
                _ => panic!(),
            }
        }
        if unit.expressions.len() == 1 {
            return unit.expressions.get(0).unwrap().clone()
        } else {
            return ExpressionKind::Unit(unit);
        }
    }

    fn parse_nullish_coalescing(pair: Pair<'_>) -> NullishCoalescing {
        let span = Self::parse_span(&pair);
        let mut expressions = vec![];
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::unit => expressions.push(Self::parse_unit(current)),
                _ => panic!()
            }
        }
        NullishCoalescing { expressions, span }
    }

    fn parse_subscript(pair: Pair<'_>) -> Subscript {
        let span = Self::parse_span(&pair);
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::expression => return Subscript { expression: Box::new(Self::parse_expression(current).kind), span },
                _ => panic!(),
            }
        }
        panic!()
    }

    fn parse_argument_list(pair: Pair<'_>) -> ArgumentList {
        let span = Self::parse_span(&pair);
        let mut arguments: Vec<Argument> = vec![];
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::argument => arguments.push(Self::parse_argument(current)),
                _ => panic!(),
            }
        }
        ArgumentList { arguments, span, resolved: false }
    }

    fn parse_group(pair: Pair<'_>) -> Group {
        let span = Self::parse_span(&pair);
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::expression => return Group { expression: Box::new(Self::parse_expression(current).kind), span },
                _ => panic!(),
            }
        }
        panic!()
    }

    fn parse_range_literal(pair: Pair<'_>) -> RangeLiteral {
        let span = Self::parse_span(&pair);
        let mut expressions: Vec<ExpressionKind> = vec![];
        let mut closed = false;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::expression => expressions.push(Self::parse_expression(current).kind),
                Rule::RANGE_OPEN => closed = false,
                Rule::RANGE_CLOSE => closed = true,
                _ => panic!(),
            }
        }
        RangeLiteral { closed, expressions, span }
    }

    fn parse_tuple_literal(pair: Pair<'_>) -> TupleLiteral {
        let span = Self::parse_span(&pair);
        let mut expressions: Vec<ExpressionKind> = vec![];
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::expression => expressions.push(Self::parse_expression(current).kind),
                _ => panic!(),
            }
        }
        TupleLiteral { expressions, span }
    }

    fn parse_array_literal(pair: Pair<'_>) -> ArrayLiteral {
        let span = Self::parse_span(&pair);
        let mut expressions: Vec<ExpressionKind> = vec![];
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::expression => expressions.push(Self::parse_expression(current).kind),
                _ => panic!(),
            }
        }
        ArrayLiteral { expressions, span }
    }

    fn parse_dictionary_literal(pair: Pair<'_>) -> DictionaryLiteral {
        panic!()
    }
    
    fn parse_type(pair: Pair<'_>) -> Type {
        let mut identifier = None;
        let mut arity = Arity::Scalar;
        let mut item_required = true;
        let mut collection_required = true;
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::COLON => {},
                Rule::identifier => identifier = Some(Self::parse_identifier(&current)),
                Rule::arity => if current.as_str() == "[]" { arity = Arity::Array; } else { arity = Arity::Dictionary; },
                Rule::optionality => {
                    if arity == Arity::Scalar {
                        item_required = false;
                    } else {
                        collection_required = false;
                    }
                },
                _ => panic!(),
            }
        }
        Type::new(
            identifier.unwrap(),
            arity,
            item_required,
            collection_required,
        )
    }

    fn parse_import_identifier_list(pair: Pair<'_>) -> Vec<Identifier> {
        let mut identifiers = vec![];
        for current in pair.into_inner() {
            match current.as_rule() {
                Rule::identifier => identifiers.push(Self::parse_identifier(&current)),
                _ => panic!(),
            }
        }
        identifiers
    }

    fn parse_identifier(pair: &Pair<'_>) -> Identifier {
        Identifier {
            name: pair.as_str().to_owned(),
            span: Self::parse_span(pair),
        }
    }

    fn parse_span(pair: &Pair<'_>) -> Span {
        let span = pair.as_span();
        Span {
            start: span.start(),
            end: span.end(),
        }
    }

    pub(crate) fn get_source_by_id(&self, id: usize) -> Option<Arc<Mutex<Source>>> {
        self.sources.get(&id).cloned()
    }
}
