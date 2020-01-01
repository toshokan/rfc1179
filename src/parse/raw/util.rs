use nom::bytes::complete::{is_a, is_not, tag};
use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::multi::separated_list;
use nom::sequence::tuple;
use nom::IResult;

pub fn whitespace_parser(i: &str) -> IResult<&str, &str> {
    is_a(" \x0B\t\x0C")(i)
}
pub fn octet_seq_parser(i: &str) -> IResult<&str, &str> {
    is_not(" \x0B\t\x0C\n")(i)
}

pub fn simple_line_parser<'a, T, F>(
    tag_text: &'a str,
    p: &'a impl Fn(&'a str) -> IResult<&'a str, F>,
    f: impl Fn(F) -> T,
) -> impl Fn(&'a str) -> IResult<&'a str, T> {
    move |i: &'a str| {
        all_consuming(tuple((tag(tag_text), p, char('\n'))))(i).map(|(r, (_, s, _))| (r, f(s)))
    }
}

pub fn list_parser(i: &str) -> IResult<&str, Vec<&str>> {
    separated_list(whitespace_parser, octet_seq_parser)(i)
}

pub fn count_parser(i: &str) -> IResult<&str, usize> {
    nom::character::complete::digit1(i).and_then(|(r, d)| {
        str::parse(d).map(|d| (r, d)).map_err(|_| {
            let error = nom::error::make_error(d, nom::error::ErrorKind::Digit);
            nom::Err::Error(error)
        })
    })
}

pub fn mini_parser<'a, T>(
    p: impl Fn(&'a str) -> IResult<&'a str, &'a str>,
    f: impl Fn(&'a str) -> T,
) -> impl Fn(&'a str) -> IResult<&'a str, T> {
    move |i: &'a str| p(i).map(|(r, x)| (r, f(x)))
}
