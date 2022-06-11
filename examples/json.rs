//! JSON parser implementation.
use somen::{call, prelude::*};
use somen_language::numeric::{float::float, signed};
use somen_language::token;
use std::collections::HashMap;

token! {
    #[derive(Clone, Debug, PartialEq)]
    enum JsonValue: JsonToken {
        Null = JsonToken::null(),
        Boolean(bool) = JsonToken::boolean(),
        Number(f64) = JsonToken::number(),
        String(String) = JsonToken::string(),
        Object(HashMap<String, JsonValue>) = JsonToken::string()
            .skip(JsonToken::symbol(Symbol::Colon))
            .and(call!(JsonValue::parser))
            .sep_by(JsonToken::symbol(Symbol::Comma), ..)
            .collect()
            .between(JsonToken::symbol(Symbol::BraceOpen), JsonToken::symbol(Symbol::BraceClose)),
        Array(Vec<JsonValue>) = call!(JsonValue::parser)
            .sep_by(JsonToken::symbol(Symbol::Comma), ..)
            .collect()
            .between(JsonToken::symbol(Symbol::BracketOpen), JsonToken::symbol(Symbol::BracketClose)),
    }
}

token! {
    #[derive(Clone, Debug, PartialEq)]
    enum JsonToken: char {
        @[match_arg = symbol]
        Symbol(Symbol) = Symbol::parser(),
        @[match = null]
        Null = tag("null"),
        @[match = boolean]
        Boolean(bool) = choice((
            tag("true").map(|_| true),
            tag("false").map(|_| false),
        )),
        @[match = number]
        Number(f64) = signed(float, false),
        @[match = string]
        String(String) = string(),
    }
}

token! {
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Symbol: char {
        BraceOpen = token('{'),
        BraceClose = token('}'),
        BracketOpen = token('['),
        BracketClose = token(']'),
        Colon = token(':'),
        Comma = token(','),
    }
}

fn spaces<'a, I: Input<Ok = char> + 'a>() -> impl Parser<I, Output = ()> + 'a {
    one_of(" \t\n\r")
        .expect("a space")
        .repeat(..)
        .discard()
        .expect("spaces")
}

fn string<'a, I: Input<Ok = char> + ?Sized + 'a>() -> impl Parser<I, Output = String> + 'a {
    choice((
        none_of("\\\""),
        token('\\').prefix(one_of("\"\\/bfnrtu")).then(|c| match c {
            '\"' => value('\"').left(),
            '\\' => value('\\').left(),
            '/' => value('/').left(),
            'b' => value('\x08').left(),
            'f' => value('\x0c').left(),
            'n' => value('\n').left(),
            'r' => value('\r').left(),
            't' => value('\t').left(),
            'u' => one_of("0123456789abcdefABCDEF")
                .expect("a hex digit")
                .times(4)
                .collect::<String>()
                .try_map(|s| {
                    char::from_u32(u32::from_str_radix(&s, 16).unwrap())
                        .ok_or("a valid unicode codepoint")
                })
                .right(),
            _ => unreachable!(),
        }),
    ))
    .repeat(..)
    .collect::<String>()
    .between(token('"'), token('"'))
}

fn main() {
    futures_executor::block_on(async {
        let mut stream = stream::from_iter(
            r#"{
                "Image": {
                    "Width": 800,
                    "Height": 600,
                    "Title":  "View from 15th Floor",
                    "Thumbnail": {
                        "Url":    "http://www.example.com/image/481989943",
                        "Height": 125,
                        "Width":  100
                    },
                    "Animated" : false,
                    "IDs": [116, 943, 234, 38793]
                },
                "escaped characters": "\u2192\"\t\r\n"
            }"#
            .chars(),
        )
        .buffered_rewind();
        let mut tokens = JsonToken::parser()
            .sep_by(spaces(), ..)
            .between(spaces(), spaces());
        let mut lexed = tokens.parse_iterable(&mut stream);
        println!(
            "{:#?}",
            JsonValue::parser().parse(&mut lexed).await.unwrap()
        );
    });
}
