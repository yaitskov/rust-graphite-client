extern crate hyper;
extern crate url;
extern crate rustc_serialize;
extern crate getopts;
extern crate rpassword;

use std::collections::HashMap;
use std::io::Read;
use hyper::Client;
use hyper::header::{Connection, Authorization, Basic, Headers};
use url::form_urlencoded;
use rustc_serialize::json::Json;
use getopts::{Options, Matches};
use std::io::Write;

const USER   : &'static str = "user";
const TARGET : &'static str = "target";
const SITE   : &'static str = "site";
const FROM   : &'static str = "from";
const TO     : &'static str = "to";
const AGG    : &'static str = "agg";

fn main() {
    let mut opts = Options::new();
    opts.usage("Aggregates graphite ticks");
    opts.reqopt("u", USER, "ldap user", "USR");
    opts.optopt("s", SITE, "url for graphite site", "URL");
    opts.optopt("t", TARGET, "graphite target expression", "EXPR");
    opts.optopt("f", FROM, "start date of graphite time series", "-7days");
    opts.optopt("e", TO, "end date of graphite time series", "");
    opts.optopt("a", AGG, "aggregator function for time series", "(sum|avg)");
    let arguments = match opts.parse(std::env::args()) {
        Ok(m) => m,
        Err(e) => panic!("Invalid arguments: {}", e)
    };

    print!("Password:");
    std::io::stdout().flush().ok().expect("Flush");
    let pass = rpassword::read_password().unwrap();
    println!("Pass is [{}]", pass);

    let client = Client::new();
    let mut headers = Headers::new();
    headers.set(Connection::close());
    headers.set(Authorization(
        Basic {
            username: arguments.opt_str(USER).unwrap(),
            password: Some(pass)
        }));
    let mut url = arguments.opt_str(SITE).unwrap_or("https://egnyte-graphite.egnyte.com/".to_owned())
        + "render?";
    let mut params = HashMap::new();
    let target = r#"groupByNode(keepLastValue(tomcat.EVENT.sjc-*.{redMessageDeliveredToClient}),3,"sumSeries")"#;
    params.insert("target", arguments.opt_str(TARGET).unwrap_or(target.to_owned()));
    params.insert("format", "json".to_owned());
    params.insert("from", arguments.opt_str(FROM).unwrap_or("-7days".to_owned()));
    if let Some(to) = arguments.opt_str(TO) {
        params.insert("to", to.to_owned());
    }
    url.push_str(&form_urlencoded::serialize(&params));
    println!("Send GET HTTP query [{}]", url); // logging to std err
    let mut res = client.get(&url)
        .headers(headers)
        .header(Connection::close())
        .send().unwrap();
    match res.status {
        hyper::Ok => {
            let mut body = String::new();
            res.read_to_string(&mut body).unwrap();
            match body.parse::<Json>() {
                Ok(jsres) => 
                    println!("{}", str_pair_vec_to_num(jsres.as_array().unwrap()[0]
                                                       .as_object().unwrap()
                                                       .get("datapoints").unwrap()
                                                       .to_owned(), choose_agg(&arguments))),
                Err(err) => panic!("Body is not json: [{}]", err)
            }            
        }
        other => {
            panic!("Bad response {}", other)
        }
    }
}

fn choose_agg(o: &Matches) -> (fn(f64, f64) -> f64) {
    match o.opt_str(AGG).unwrap_or("sum".to_owned()).as_ref() {
        "sum" => sum,
        "avg" => avg,
        x => panic!("Unsupported {}", x)
    }
}

fn sum(s: f64, x: f64) -> f64 {
    s + x
}

fn avg(s: f64, x: f64) -> f64 {
    (s + x) / 2.0
}

fn str_pair_vec_to_num(data_points: Json, agg: fn(f64, f64) -> f64) -> f64 {
    let mut itr = data_points.as_array().unwrap()
        .iter()
        .map(|js_pair| match js_pair.as_array().unwrap()[0].as_f64() {
            Some(n) => { n }
            None => { std::f64::NAN }
        })
        .filter(|x| !x.is_nan());
    return match itr.next() {
        Some(first) => itr.fold(first, &agg),
        None => std::f64::NAN
    }
}


