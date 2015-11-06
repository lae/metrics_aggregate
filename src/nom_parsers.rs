use nom::{IResult};
use nom::IResult::*;

use std::fmt;
use std::str;
use std::str::FromStr;
//use std::collections::HashMap;

#[derive(Debug)]
enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Unknown
}

fn is_token(c: u8) -> bool {
    // Matches 0-9A-Za-z_
    (c > 47 && c < 58) || (c > 64 && c < 91) || c == 95 || (c > 96 && c < 123)
}

fn is_not_line_ending(c: u8) -> bool { c != b'\r' && c != b'\n' }

struct MetricDescriptionLine<'a> {
    name: &'a [u8],
    value: &'a [u8]
}

struct MetricTypeLine<'a> {
    name: &'a [u8],
    value: MetricType
}

struct MetricDataLine<'a> {
    name: &'a [u8],
    parameters: &'a [u8],
    value: f64
}

struct MetricGaugeLine<'a> {
    name: &'a [u8],
    value: f64
}

struct MetricHistogramSumLine<'a> {
    name: &'a [u8],
    value: f64
}

struct MetricHistogramCountLine<'a> {
    name: &'a [u8],
    value: f64
}

struct MetricHistogramBucketLine<'a> {
    name: &'a [u8],
    parameters: &'a [u8],
    value: f64
}

impl<'a> fmt::Debug for MetricDescriptionLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = str::from_utf8(self.name).unwrap_or("");
        let value = str::from_utf8(self.value).unwrap_or("");
        write!(f, "MetricDescriptionLine {{ name: {:?}, value: {:?} }}", name, value)
    }
}

impl<'a> fmt::Debug for MetricTypeLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = str::from_utf8(self.name).unwrap_or("");
        write!(f, "MetricTypeLine {{ name: {:?}, value: {:?} }}", name, self.value)
    }
}

impl<'a> fmt::Debug for MetricDataLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = str::from_utf8(self.name).unwrap_or("");
        let parameters = str::from_utf8(self.parameters).unwrap_or("");
        write!(f, "MetricDataLine {{ name: {:?}, parameters: {:?}, value: {:?} }}", name, parameters, self.value)
    }
}

impl<'a> fmt::Debug for MetricGaugeLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = str::from_utf8(self.name).unwrap_or("");
        write!(f, "MetricGaugeLine {{ name: {:?}, value: {:?} }}", name, self.value)
    }
}

impl<'a> fmt::Debug for MetricHistogramSumLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = str::from_utf8(self.name).unwrap_or("");
        write!(f, "MetricHistogramSumLine {{ name: {:?}, value: {:?} }}", name, self.value)
    }
}

impl<'a> fmt::Debug for MetricHistogramCountLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = str::from_utf8(self.name).unwrap_or("");
        write!(f, "MetricHistogramCountLine {{ name: {:?}, value: {:?} }}", name, self.value)
    }
}

impl<'a> fmt::Debug for MetricHistogramBucketLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = str::from_utf8(self.name).unwrap_or("");
        let parameters = str::from_utf8(self.parameters).unwrap_or("");
        write!(f, "MetricHistogramBucketLine {{ name: {:?}, parameters: {:?}, value: {:?} }}", name, parameters, self.value)
    }
}

named!(parse_single_line, take_until_and_consume!("\n"));

named!(parse_metric_description<&[u8], MetricDescriptionLine>,
    chain!(
            tag!("# HELP ") ~
        n:  take_while1!(is_token) ~
            tag!(" ") ~
        v:  take_while1!(is_not_line_ending),
        || {
            MetricDescriptionLine {
                name: n,
                value: v,
            }
        }
    )
);

named!(parse_metric_type<&[u8], MetricTypeLine>,
    chain!(
            tag!("# TYPE ") ~
        n:  take_while1!(is_token) ~
            tag!(" ") ~
        v:  take_while1!(is_not_line_ending),
        || {
            MetricTypeLine {
                name: n,
                value: match v {
                    b"counter" => MetricType::Counter,
                    b"gauge" => MetricType::Gauge,
                    b"histogram" => MetricType::Histogram,
                    b"summary" => MetricType::Summary,
                    _ => MetricType::Unknown
                },
            }
        }
    )
);

named!(parse_metric_histogram_count<&[u8], MetricHistogramCountLine>,
    chain!(
        n:  take_until!("_count ") ~
            tag!("_count ") ~
        v:  map_res!(
                map_res!(
                    take_while1!(is_not_line_ending),
                    str::from_utf8
                ),
                FromStr::from_str
            ),
        || {
            MetricHistogramCountLine {
                name: n,
                value: v,
            }
        }
    )
);
named!(parse_metric_histogram_sum<&[u8], MetricHistogramSumLine>,
    chain!(
        n:  take_until!("_sum ") ~
            tag!("_sum ") ~
        v:  map_res!(
                map_res!(
                    take_while1!(is_not_line_ending),
                    str::from_utf8
                ),
                FromStr::from_str
            ),
        || {
            MetricHistogramSumLine {
                name: n,
                value: v,
            }
        }
    )
);
named!(parse_metric_histogram_bucket<&[u8], MetricHistogramBucketLine>,
    chain!(
        n:  take_until!("_bucket{") ~
            tag!("_bucket{") ~
        p:  take_until!("}") ~
            tag!("} ") ~
        v:  map_res!(
                map_res!(
                    take_while1!(is_not_line_ending),
                    str::from_utf8
                ),
                FromStr::from_str
            ),
        || {
            MetricHistogramBucketLine {
                name: n,
                parameters: p,
                value: v,
            }
        }
    )
);

named!(parse_metric_gauge<&[u8], MetricGaugeLine>,
    chain!(
        n:  take_while1!(is_token) ~
            tag!(" ") ~
        v:  map_res!(
                map_res!(
                    take_while1!(is_not_line_ending),
                    str::from_utf8
                ),
                FromStr::from_str
            ),
        || {
            MetricGaugeLine {
                name: n,
                value: v,
            }
        }
    )
);

named!(parse_metric_data<&[u8], MetricDataLine>,
    chain!(
        n:  take_while1!(is_token) ~
            tag!("{") ~
        p:  take_until!("}") ~
            tag!("} ") ~
        v:  map_res!(
                map_res!(
                    take_while1!(is_not_line_ending),
                    str::from_utf8
                ),
                FromStr::from_str
            ),
        || {
            MetricDataLine {
                name: n,
                parameters: p,
                value: v,
            }
        }
    )
);

pub fn parse(data: &[u8]) {
    let mut buf = &data[..];
    loop {
        if let IResult::Done(remaining, line) = parse_single_line(buf) {
            if let IResult::Done(_, metric_line) = parse_metric_description(line) {
                println!("{:?}", metric_line);
            }
            else if let IResult::Done(_, metric_line) = parse_metric_type(line) {
                println!("{:?}", metric_line);
            }
            else if let IResult::Done(_, metric_line) = parse_metric_histogram_sum(line) {
                println!("{:?}", metric_line);
            }
            else if let IResult::Done(_, metric_line) = parse_metric_histogram_count(line) {
                println!("{:?}", metric_line);
            }
            else if let IResult::Done(_, metric_line) = parse_metric_histogram_bucket(line) {
                println!("{:?}", metric_line);
            }
            else if let IResult::Done(_, metric_line) = parse_metric_data(line) {
                println!("{:?}", metric_line);
            }
            else if let IResult::Done(_, metric_line) = parse_metric_gauge(line) {
                println!("{:?}", metric_line);
            }
            buf = remaining;
        }
        else { panic!("end???"); }
        if buf.is_empty() { break }
    }
}
