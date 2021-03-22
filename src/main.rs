use chrono::prelude::*;
use colored::*;
use rayon::prelude::*;
use reqwest;
use select::document::Document;
use select::predicate::{Name, Predicate, Text};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::{fs, thread, time};

#[derive(Serialize, Deserialize, Debug)]
struct Retailer {
    name: String,
    product: String,
    url: String,
    no_stock_search_term: String,
}

impl Retailer {
    fn perform_scan(&self, client: reqwest::blocking::Client) {
        let mut res = client.get(self.url.clone()).send().unwrap();
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        let document = Document::from(body.as_str());

        let results: Vec<select::node::Node> = document
            .find(Name("body").descendant(Text))
            .filter(|node| node.text().trim().contains(&self.no_stock_search_term))
            .collect();

        let local_time: DateTime<Local> = Local::now();

        if results.len() < 1 {
            println!(
                "{} !!!! {} Stock detected at Retailer {} {}",
                local_time.format("%T").to_string().bright_yellow(),
                &self.name.bright_green(),
                &self.product.bright_cyan(),
                &self.url
            );
        } else {
            println!(
                "{} {} no stock ({})",
                local_time.format("%T").to_string().bright_yellow(),
                &self.name.bright_green(),
                &self.product.bright_cyan(),
            );
        }
    }
}

fn get_retailers() -> rayon::vec::IntoIter<Retailer> {
    let retailer_json =
        fs::read_to_string("./retailers.json").expect("error: retailers.json not provided");
    let retailers: Vec<Retailer> = serde_json::from_str(&retailer_json).unwrap();
    retailers.into_par_iter()
}

fn main() {
    const SLEEP_SECS: u64 = 10;
    let client = reqwest::blocking::Client::new();
    loop {
        let retailers = get_retailers();
        println!("Starting scan batch");
        retailers.for_each(|retailer| {
            retailer.perform_scan(client.clone());
        });
        println!(
            "Finished scan batch, sleeping for {} seconds \n",
            SLEEP_SECS
        );
        thread::sleep(time::Duration::from_secs(SLEEP_SECS));
    }
}
