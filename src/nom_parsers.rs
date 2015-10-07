use nom::{IResult};
use nom::IResult::*;

use std::str::from_utf8;
use std::fmt;

pub struct MetricMetadata<'m> {
    pub name: &'m [u8],
    pub metric_type: &'m [u8],
    pub description: &'m [u8]
}

impl<'m> fmt::Debug for MetricMetadata<'m> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = from_utf8(self.name).unwrap_or("");
        let metric_type = from_utf8(self.metric_type).unwrap_or("");
        let description = from_utf8(self.description).unwrap_or("");
        write!(f, "MetricMetadata ( name: '{}', metric_type: '{}', description: '{}' )", name, metric_type, description)
    }
}

fn is_metric_name(c: u8) -> bool {
    // Matches 0-9A-Za-z_
    (c > 47 && c < 58) || (c > 64 && c < 91) || c == 95 || (c > 96 && c < 123)
}

fn not_line_ending(c: u8) -> bool {
    c != b'\r' && c != b'\n'
}

named!(line_ending, alt!(tag!("\n") | tag!("\r\n")));

named!(metric_metadata(&'a [u8]) -> MetricMetadata<'a>,
    chain!(
            take_until!("#")        ~
            tag!("# HELP ")         ~
        name: take_while1!(is_metric_name)      ~
            tag!(" ")               ~
        description: take_while1!(not_line_ending) ~
            line_ending             ~
            tag!("# TYPE ")         ~
            take_until!(" ")        ~
            tag!(" ")               ~
        metric_type: take_while1!(not_line_ending) ~
            line_ending,

        || MetricMetadata {
            name: name,
            description: description,
            metric_type: metric_type,
        }
    )
);

pub fn parse(data: &[u8]) {
    let mut buf = &data[..];
    loop {
        match metric_metadata(buf) {
            IResult::Done(b, a) => {
                println!("{:?}", a);
                buf = b;
                if b.is_empty() {
                    break;
                }
            },
            IResult::Error(_err) => return,
            IResult::Incomplete(inc) => panic!("incomplete: {:?}", inc)
        }
    }
}
