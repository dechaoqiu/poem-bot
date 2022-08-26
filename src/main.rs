// use std::collections::HashSet;
use std::sync::Arc;

use std::env;
// use tokio::io::{self, AsyncReadExt};

use futures::future::join_all;
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
    sync::Mutex,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Clause {
    #[serde(rename = "Content")]
    content: String,
    #[serde(rename = "TonesSpecified")]
    tones_specified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Poem {
    #[serde(rename = "Author")]
    author: Option<String>,
    #[serde(rename = "AuthorId")]
    author_id: Option<u32>,
    #[serde(rename = "AuthorIdSpecified")]
    author_id_specified: Option<bool>,
    #[serde(rename = "Dynasty")]
    dynasty: Option<String>,
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "GroupIndex")]
    group_index: Option<u32>,
    #[serde(rename = "GroupIndexSpecified")]
    group_index_specified: Option<bool>,
    #[serde(rename = "IsTwoClausesPerSentence")]
    is_two_clauses_per_sentence: Option<bool>,
    #[serde(rename = "IsTwoClausesPerSentenceSpecified")]
    is_two_clauses_per_sentence_specified: Option<bool>,
    #[serde(rename = "Note")]
    note: Option<String>,
    #[serde(rename = "Preface")]
    preface: Option<String>,
    #[serde(rename = "Rhyme")]
    rhyme: Option<String>,
    #[serde(rename = "TuneIdSpecified")]
    tune_id_specified: Option<bool>,
    #[serde(rename = "Type")]
    poem_type: Option<String>,
    #[serde(rename = "TypeDetail")]
    type_detail: Option<String>,
    #[serde(rename = "Clauses")]
    clauses: Vec<Clause>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Content {
    #[serde(rename = "Poem")]
    poem: Poem,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    #[serde(rename = "ShiData")]
    shi_data: Vec<Poem>,
}

async fn fetch_poem(
    client: &reqwest::Client,
    index: u32,
) -> Result<Response, Box<dyn std::error::Error>> {
    println!("fetching poem: {}", index);
    let data = client
        .get(format!(
            "https://api.sou-yun.cn/open/poem?dynasty=Tang&key={}&type=poem&jsontype=true",
            index
        ))
        .header("accept", "application/json, text/javascript, */*; q=0.01")
        .send()
        .await?
        .json::<Response>()
        .await?;
    Ok(data)
}

async fn fetch_data(
    batch: u32,
    file_arc: Arc<Mutex<File>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("fetching data for batch: {}", batch);
    let client = reqwest::Client::new();

    // let mut downloaded_ids = HashSet::<u32>::new();
    // let file_path = format!(
    //     "{}/ids.txt",
    //     env::var("OUTPUT_FOLDER").expect("Must specify ENV: OUTPUT_FOLDER")
    // );
    // let mut f = File::open(file_path).await?;
    // let mut buffer: Vec<u8> = Vec::new();
    // f.read_to_end(&mut buffer).await?;
    // let s = match std::str::from_utf8(&buffer[..]) {
    //     Ok(v) => {
    //         let mut lines = v.lines();
    //         let ids: HashSet<u32> = lines.map(|line| line.parse::<u32>().unwrap() ).collect();
    //         downloaded_ids = ids;
    //     },
    //     Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    // };

    for offset in 0..10000 {
        let id = batch * 10000 + offset;
        if id == 0 {
            continue;
        }
        // if downloaded_ids.contains(&id) {
        //     println!("skipping id: {:?}", id);
        //     continue;
        // }
        let resp = fetch_poem(&client, id).await;
        if resp.is_err() {
            eprintln!("Error fetching: {} {:?}", id, resp);
            continue;
        }
        let resp = resp.unwrap();
        let data = format!("{}\n", &serde_json::to_string(&resp).unwrap());
        let res = file_arc.lock().await.write_all(data.as_bytes()).await;
        if let Err(e) = res {
            eprintln!("Couldn't write to file: {}", e);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = format!(
        "{}/poem.txt",
        env::var("OUTPUT_FOLDER").expect("Must specify ENV: OUTPUT_FOLDER")
    );
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)
        .await
        .unwrap();
    let file_arc = Arc::new(Mutex::new(file));

    let mut handlers = Vec::new();

    for batch in 0..110 {
        let fut = fetch_data(batch, file_arc.clone());
        handlers.push(fut);
    }
    join_all(handlers).await;
    Ok(())
}
