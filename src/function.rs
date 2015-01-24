use std::{error,fmt};
use std::num::Float;

use super::{EvaluationContext,Functions,Value};

pub trait Function {
    fn evaluate<'a, 'd>(&self,
                        context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>;
}

#[derive(Copy,Clone,Debug,PartialEq,Hash)]
pub enum ArgumentType {
    Nodeset,
    Boolean,
    Number,
    String,
}

#[derive(Copy,Clone,Debug,PartialEq,Hash)]
pub enum Error {
    TooManyArguments{ expected: usize, actual: usize },
    NotEnoughArguments{ expected: usize, actual: usize },
    WrongType{ expected: ArgumentType, actual: ArgumentType },
}

impl Error {
    fn wrong_type(actual: &Value, expected: ArgumentType) -> Error {
        let actual = match *actual {
            Value::Nodes(..)   => ArgumentType::Nodeset,
            Value::String(..)  => ArgumentType::String,
            Value::Number(..)  => ArgumentType::Number,
            Value::Boolean(..) => ArgumentType::Boolean,
        };

        Error::WrongType {
            expected: expected,
            actual: actual
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            TooManyArguments{..}   => "too many arguments",
            NotEnoughArguments{..} => "not enough arguments",
            WrongType{..}          => "argument of wrong type",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Error::*;
        match *self {
            TooManyArguments{expected, actual} => {
                write!(fmt, "too many arguments, expected {} but had {}", expected, actual)
            },
            NotEnoughArguments{expected, actual} => {
                write!(fmt, "not enough arguments, expected {} but had {}", expected, actual)
            },
            WrongType{expected, actual} => {
                write!(fmt, "argument was the wrong type, expected {:?} but had {:?}", expected, actual)
            },
        }
    }
}

fn minimum_arg_count<T>(args: &Vec<T>, minimum: usize) -> Result<(), Error> {
    let actual = args.len();
    if actual < minimum {
        Err(Error::NotEnoughArguments{expected: minimum, actual: actual})
    } else {
        Ok(())
    }
}

fn exact_arg_count<T>(args: &Vec<T>, expected: usize) -> Result<(), Error> {
    let actual = args.len();
    if actual < expected {
        Err(Error::NotEnoughArguments{ expected: expected, actual: actual })
    } else if actual > expected {
        Err(Error::TooManyArguments{ expected: expected, actual: actual })
    } else {
        Ok(())
    }
}

fn string_args(args: Vec<Value>) -> Result<Vec<String>, Error> {
    fn string_arg(v: Value) -> Result<String, Error> {
        match v {
            Value::String(s) => Ok(s),
            _ => Err(Error::wrong_type(&v, ArgumentType::String)),
        }
    }

    args.into_iter().map(string_arg).collect()
}

fn one_number(args: Vec<Value>) -> Result<f64, Error> {
    match &args[0] {
        &Value::Number(v) => Ok(v),
        a => Err(Error::wrong_type(a, ArgumentType::Number)),
    }
}

struct Last;

impl Function for Last {
    fn evaluate<'a, 'd>(&self,
                        context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 0));
        Ok(Value::Number(context.size() as f64))
    }
}

struct Position;

impl Function for Position {
    fn evaluate<'a, 'd>(&self,
                        context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 0));
        Ok(Value::Number(context.position() as f64))
    }
}

struct Count;

impl Function for Count {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 1));
        let arg = &args[0];
        match arg {
            &Value::Nodes(ref nodeset) => Ok(Value::Number(nodeset.size() as f64)),
            _ => Err(Error::wrong_type(arg, ArgumentType::Nodeset)),
        }
    }
}

struct Concat;

impl Function for Concat {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(minimum_arg_count(&args, 2));
        let args = try!(string_args(args));
        Ok(Value::String(args.concat()))
    }
}

struct TwoStringPredicate(fn(&str, &str) -> bool);

impl Function for TwoStringPredicate {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 2));
        let args = try!(string_args(args));
        let v = self.0(&*args[0], &*args[1]);
        Ok(Value::Boolean(v))
    }
}

fn starts_with() -> TwoStringPredicate { TwoStringPredicate(StrExt::starts_with) }
fn contains() -> TwoStringPredicate { TwoStringPredicate(StrExt::contains) }

struct Substring(for<'s> fn(&'s str, &'s str) -> &'s str);

impl Function for Substring {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 2));
        let args = try!(string_args(args));
        let s = self.0(&*args[0], &*args[1]);
        Ok(Value::String(s.to_string()))
    }
}

fn substring_before() -> Substring {
    fn inner<'a>(haystack: &'a str, needle: &'a str) -> &'a str {
        match haystack.find_str(needle) {
            Some(pos) => &haystack[..pos],
            None => "",
        }
    }
    Substring(inner)
}

fn substring_after() -> Substring {
    fn inner<'a>(haystack: &'a str, needle: &'a str) -> &'a str {
        match haystack.find_str(needle) {
            Some(pos) => &haystack[pos + needle.len()..],
            None => "",
        }
    }
    Substring(inner)
}

struct Not;

impl Function for Not {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 1));
        let arg = &args[0];
        match arg {
            &Value::Boolean(v) => Ok(Value::Boolean(!v)),
            _ => Err(Error::wrong_type(arg, ArgumentType::Boolean)),
        }
    }
}

struct BooleanLiteral(bool);

impl Function for BooleanLiteral {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 0));
        Ok(Value::Boolean(self.0))
    }
}

fn true_fn() -> BooleanLiteral { BooleanLiteral(true) }
fn false_fn() -> BooleanLiteral { BooleanLiteral(false) }

struct NumberConvert(fn(f64) -> f64);

impl Function for NumberConvert {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 1));
        let arg = try!(one_number(args));
        Ok(Value::Number(self.0(arg)))
    }
}

fn floor() -> NumberConvert { NumberConvert(Float::floor) }
fn ceiling() -> NumberConvert { NumberConvert(Float::ceil) }

pub fn register_core_functions(functions: &mut Functions) {
    functions.insert("last".to_string(), box Last);
    functions.insert("position".to_string(), box Position);
    functions.insert("count".to_string(), box Count);
    functions.insert("concat".to_string(), box Concat);
    functions.insert("starts-with".to_string(), box starts_with());
    functions.insert("contains".to_string(), box contains());
    functions.insert("substring-before".to_string(), box substring_before());
    functions.insert("substring-after".to_string(), box substring_after());
    functions.insert("not".to_string(), box Not);
    functions.insert("true".to_string(), box true_fn());
    functions.insert("false".to_string(), box false_fn());
    functions.insert("floor".to_string(), box floor());
    functions.insert("ceiling".to_string(), box ceiling());
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use document::Package;
    use super::super::{EvaluationContext,LiteralValue,Value,Functions,Variables,Namespaces};
    use super::super::nodeset::ToNode;
    use super::{
        Function,
        Error,
        Last,
        Position,
        Count,
        Concat,
    };

    struct Setup<'d> {
        functions: Functions,
        variables: Variables<'d>,
        namespaces: Namespaces,
    }

    impl<'d> Setup<'d> {
        fn new() -> Setup<'d> {
            Setup {
                functions: HashMap::new(),
                variables: HashMap::new(),
                namespaces: HashMap::new(),
            }
        }

        fn evaluate<N, F>(&self, node: N, f: F, args: Vec<Value<'d>>)
            -> Result<Value<'d>, Error>
            where N: ToNode<'d>,
                  F: Function
        {
            let context = EvaluationContext::new(
                node, &self.functions, &self.variables, &self.namespaces
            );
            f.evaluate(&context, args)
        }
    }

    fn evaluate_literal<F>(f: F, args: Vec<LiteralValue>) -> Result<LiteralValue, Error>
        where F: Function
    {
        let package = Package::new();
        let doc = package.as_document();
        let setup = Setup::new();

        let args = args.into_iter().map(|a| a.into_value()).collect();

        let r = setup.evaluate(doc.root(), f, args);

        r.map(|r| r.into_literal_value())
    }

    #[test]
    fn last_returns_context_size() {
        let r = evaluate_literal(Last, vec![]);
        assert_eq!(Ok(LiteralValue::Number(1.0)), r);
    }

    #[test]
    fn position_returns_context_position() {
        let r = evaluate_literal(Position, vec![]);

        assert_eq!(Ok(LiteralValue::Number(1.0)), r);
    }

    #[test]
    fn count_counts_nodes_in_nodeset() {
        let package = Package::new();
        let doc = package.as_document();
        let setup = Setup::new();

        let nodeset = nodeset![doc.root()];
        let r = setup.evaluate(doc.root(), Count, vec![Value::Nodes(nodeset)]);

        assert_eq!(Ok(Value::Number(1.0)), r);
    }

    #[test]
    fn concat_combines_strings() {
        let args = vec![LiteralValue::String("hello".to_string()),
                        LiteralValue::String(" ".to_string()),
                        LiteralValue::String("world".to_string())];
        let r = evaluate_literal(Concat, args);

        assert_eq!(Ok(LiteralValue::String("hello world".to_string())), r);
    }

    #[test]
    fn starts_with_checks_prefixes() {
        let args = vec![LiteralValue::String("hello".to_string()),
                        LiteralValue::String("he".to_string())];
        let r = evaluate_literal(super::starts_with(), args);

        assert_eq!(Ok(LiteralValue::Boolean(true)), r);
    }

    #[test]
    fn contains_looks_for_a_needle() {
        let args = vec![LiteralValue::String("astronomer".to_string()),
                        LiteralValue::String("ono".to_string())];
        let r = evaluate_literal(super::contains(), args);

        assert_eq!(Ok(LiteralValue::Boolean(true)), r);
    }

    #[test]
    fn substring_before_slices_before() {
        let args = vec![LiteralValue::String("1999/04/01".to_string()),
                        LiteralValue::String("/".to_string())];
        let r = evaluate_literal(super::substring_before(), args);

        assert_eq!(Ok(LiteralValue::String("1999".to_string())), r);
    }

    #[test]
    fn substring_after_slices_after() {
        let args = vec![LiteralValue::String("1999/04/01".to_string()),
                        LiteralValue::String("/".to_string())];
        let r = evaluate_literal(super::substring_after(), args);

        assert_eq!(Ok(LiteralValue::String("04/01".to_string())), r);
    }

    #[test]
    fn floor_rounds_down() {
        let r = evaluate_literal(super::floor(), vec![LiteralValue::Number(199.99)]);

        assert_eq!(Ok(LiteralValue::Number(199.0)), r);
    }

    #[test]
    fn ceiling_rounds_up() {
        let r = evaluate_literal(super::ceiling(), vec![LiteralValue::Number(199.99)]);

        assert_eq!(Ok(LiteralValue::Number(200.0)), r);
    }
}
