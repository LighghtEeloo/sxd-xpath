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

struct StartsWith;

impl Function for StartsWith {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 2));
        let args = try!(string_args(args));
        let v = args[0].starts_with(&*args[1]);
        Ok(Value::Boolean(v))
    }
}

struct Contains;

impl Function for Contains {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 2));
        let args = try!(string_args(args));
        let v = args[0].contains(&*args[1]);
        Ok(Value::Boolean(v))
    }
}

struct SubstringBefore;

impl Function for SubstringBefore {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 2));
        let args = try!(string_args(args));
        let haystack = &args[0];

        let s = match haystack.find_str(&*args[1]) {
            Some(pos) => &haystack[..pos],
            None => "",
        };

        Ok(Value::String(s.to_string()))
    }
}

struct SubstringAfter;

impl Function for SubstringAfter {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 2));
        let args = try!(string_args(args));
        let haystack = &args[0];
        let needle = &*args[1];

        let s = match haystack.find_str(needle) {
            Some(pos) => &haystack[pos + needle.len()..],
            None => "",
        };

        Ok(Value::String(s.to_string()))
    }
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

struct True;

impl Function for True {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 0));
        Ok(Value::Boolean(true))
    }
}

struct False;

impl Function for False {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 0));
        Ok(Value::Boolean(false))
    }
}

struct Floor;

impl Function for Floor {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 1));
        let arg = try!(one_number(args));
        Ok(Value::Number(arg.floor()))
    }
}

struct Ceiling;

impl Function for Ceiling {
    fn evaluate<'a, 'd>(&self,
                        _context: &EvaluationContext<'a, 'd>,
                        args: Vec<Value<'d>>) -> Result<Value<'d>, Error>
    {
        try!(exact_arg_count(&args, 1));
        let arg = try!(one_number(args));
        Ok(Value::Number(arg.ceil()))
    }
}

pub fn register_core_functions(functions: &mut Functions) {
    functions.insert("last".to_string(), box Last);
    functions.insert("position".to_string(), box Position);
    functions.insert("count".to_string(), box Count);
    functions.insert("concat".to_string(), box Concat);
    functions.insert("starts-with".to_string(), box StartsWith);
    functions.insert("contains".to_string(), box Contains);
    functions.insert("substring-before".to_string(), box SubstringBefore);
    functions.insert("substring-after".to_string(), box SubstringAfter);
    functions.insert("not".to_string(), box Not);
    functions.insert("true".to_string(), box True);
    functions.insert("false".to_string(), box False);
    functions.insert("floor".to_string(), box Floor);
    functions.insert("ceiling".to_string(), box Ceiling);
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
        StartsWith,
        Contains,
        SubstringBefore,
        SubstringAfter,
        Floor,
        Ceiling,
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
        let r = evaluate_literal(StartsWith, args);

        assert_eq!(Ok(LiteralValue::Boolean(true)), r);
    }

    #[test]
    fn contains_looks_for_a_needle() {
        let args = vec![LiteralValue::String("astronomer".to_string()),
                        LiteralValue::String("ono".to_string())];
        let r = evaluate_literal(Contains, args);

        assert_eq!(Ok(LiteralValue::Boolean(true)), r);
    }

    #[test]
    fn substring_before_slices_before() {
        let args = vec![LiteralValue::String("1999/04/01".to_string()),
                        LiteralValue::String("/".to_string())];
        let r = evaluate_literal(SubstringBefore, args);

        assert_eq!(Ok(LiteralValue::String("1999".to_string())), r);
    }

    #[test]
    fn substring_after_slices_after() {
        let args = vec![LiteralValue::String("1999/04/01".to_string()),
                        LiteralValue::String("/".to_string())];
        let r = evaluate_literal(SubstringAfter, args);

        assert_eq!(Ok(LiteralValue::String("04/01".to_string())), r);
    }

    #[test]
    fn floor_rounds_down() {
        let r = evaluate_literal(Floor, vec![LiteralValue::Number(199.99)]);

        assert_eq!(Ok(LiteralValue::Number(199.0)), r);
    }

    #[test]
    fn ceiling_rounds_up() {
        let r = evaluate_literal(Ceiling, vec![LiteralValue::Number(199.99)]);

        assert_eq!(Ok(LiteralValue::Number(200.0)), r);
    }
}
