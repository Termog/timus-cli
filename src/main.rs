extern crate getopts;
extern crate confy;
use serde::{Serialize, Deserialize};
use scraper::{Html, Selector};
use reqwest;
use getopts::Options;
use std::collections::HashMap;
use std::env;
use std::fs;
use tokio::time::{sleep, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    id: String,
    compiler: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self { Self { id: "".into(), compiler: "67".into() } }
}

fn print_usage(program: &str, opts: Options) {
        let brief = format!("Usage: {} [options] FILE", program);
            eprint!("{}", opts.usage(&brief));
}

async fn subm(problem: &str, source: &str, id: &str, compiler: &str) {
        let mut map = HashMap::new();
        map.insert("Action","submit");
        map.insert("SpaceID","1");
        map.insert("JudgeID",&id);
        map.insert("Language",&compiler);
        map.insert("ProblemNum",&problem);
        map.insert("Source",&source);

        let client = reqwest::Client::new();
        let _res = client.post("https://acm.timus.ru/submit.aspx")
            .form(&map)
            .send()
            .await;
}

async fn result(id: &str) {
    loop {
        let link = "https://acm.timus.ru/status.aspx?author=".to_string();
        let link = format!("{}{}", link, id);
        let rep = reqwest::get(link)
            .await
            .unwrap()
            .text()
            .await;
        let text;
        match rep {
            Ok(ref x) => text = x,
            Err(_) => {
                eprintln!("Could not get information from acm.timus.ru");
                return;
            }
        }
        let fragment = Html::parse_fragment(text);
        let _doc = Html::parse_document(&text);
        let selector = Selector::parse("tr.even").unwrap();
        let element = fragment.select(&selector).next().unwrap();
        let resp = element.text().collect::<Vec<_>>();
        if resp[7] != "Compiling" && resp[7] != "Running"{
            display(resp);
            break;
        }
       sleep(Duration::from_millis(100)).await;
    }
}

fn display( resp: Vec<&str> ) {
    println!("   Compiler: {}",resp[6]);
    println!("   {}",resp[7]);
    if resp[7] == "Accepted" {
        println!("   Time: {}",resp[8]);
        println!("   Memory: {}",resp[9]);
    } else if resp[7] != "Compilation error" {
        println!("   Test: {}",resp[8]);
        println!("   Time: {}",resp[9]);
        println!("   Memory: {}",resp[10]);
    }
}

fn main() -> Result<(), confy::ConfyError> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("p","","set problem number", "NUMBER");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!("{}",f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }
    let problem = matches.opt_str("p");
    let file = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return Ok(());
    };
    let pnum;
    match problem {
        Some(ref x) => pnum = x,
        None => {
            eprintln!("Problem number not specified/n");
            print_usage(&program,opts);
            return Ok(());
        }
    }
    let cfg: Config = confy::load("timus")?;
    let source = fs::read_to_string(file)
        .expect("Can't open the file");
    if cfg.id == "" {
        eprintln!("timus id is not set in the config file!");
        return Ok(());
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = subm(&pnum,&source,&cfg.id,&cfg.compiler);
    rt.block_on(future);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let future2 = result(&cfg.id[0..6]);
    rt.block_on(future2);
    Ok(())
}
