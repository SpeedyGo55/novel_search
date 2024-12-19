// A CLI-tool to browse books from the Open Library API
// Features: search by name, search by author, search by ISBN, search by text, random book, top book by genre

use reqwest;
use clap::{Parser, Subcommand};
use serde_json::Value;
use rand::random;

#[derive(Parser, Debug)]
#[command(author = "SpeedyGo55", version, about = "A simple CLI-tool to browse books from the Open Library API", name = "novel_search")]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Search for books by name, ISBN, or text
    Search(Search),
    /// Get a random book from a genre
    Random(Random),
}

#[derive(Parser, Debug, Clone)]
struct Search {
    /// The search query. Can be a name, ISBN, or text
    #[arg(required = true)]
    data: String,
    /// The type of search query
    #[command(subcommand)]
    search_type: SearchType,
    /// The number of results to return (not applicable for ISBN search)
    #[arg(short, long, default_value = "2")]
    limit: i32,
}

#[derive(Parser, Debug, Clone)]
struct Random {
    /// The genre to get a random book from
    genre: String,
}

#[derive(Subcommand, Debug, Clone)]
enum SearchType {
    /// Search by name
    Name,
    /// Search by ISBN
    ISBN,
    /// Search by subject
    Subject,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.cmd {
        Commands::Search(search) => {
            match search.search_type {
                SearchType::Name => {
                    let api_response = search_name(&search.data, search.limit).await;
                    display_books(api_response);
                },
                SearchType::ISBN => {
                    let api_response = search_isbn(&search.data).await;
                    display_isbn_books(api_response);
                },
                SearchType::Subject => {
                    let api_response = search_subject(&search.data, search.limit).await;
                    display_subject_titles(api_response);
                }
            }
        },
        Commands::Random(random) => {
            let api_response = random_book(&random.genre).await;
            let title = get_random_book_title(api_response);
            let api_response = search_name(&title, 1).await;
            display_books(api_response);
        }
    }

}

async fn search_name(name: &str, limit: i32) -> Value {
    let url = format!("https://openlibrary.org/search.json?title={}&limit={}", name, limit);
    let response = reqwest::get(&url).await;
    let response = match response {
        Ok(response) => response,
        Err(_) => {
            println!("No books found with the name: {}", name);
            std::process::exit(1);
        }
    };
    let api_response = response.json().await;
    let api_response = match api_response {
        Ok(api_response) => api_response,
        Err(_) => {
            println!("No books found with the name: {}", name);
            std::process::exit(1);
        }
    };
    api_response
}

async fn search_isbn(isbn: &str) -> Value {
    let url = format!("https://openlibrary.org/api/volumes/brief/isbn/{}.json", isbn);
    let response = reqwest::get(&url).await;
    let response = match response {
        Ok(response) => response,
        Err(_) => {
            println!("No books found with the ISBN: {}", isbn);
            std::process::exit(1);
        }
    };
    let api_response = response.json().await;
    let api_response = match api_response {
        Ok(api_response) => api_response,
        Err(_) => {
            println!("No books found with the ISBN: {}", isbn);
            std::process::exit(1);
        }
    };
    api_response
}

async fn search_subject(subject: &str, limit: i32) -> Value {
    let url = format!("https://openlibrary.org/subjects/{}.json?limit={}", subject.to_lowercase(), limit);
    let response = reqwest::get(&url).await;
    let response = match response {
        Ok(response) => response,
        Err(_) => {
            println!("No books found with the subject: {}", subject);
            std::process::exit(1);
        }
    };
    let text = response.text().await;
    let api_response = serde_json::from_str(&text.unwrap());
    let api_response = match api_response {
        Ok(api_response) => api_response,
        Err(e) => {
            println!("No books found with the subject: {}", subject);
            println!("{}", e);
            std::process::exit(1);
        }
    };
    api_response
}

async fn random_book(genre: &str) -> Value {
    let limit = random::<u8>();
    let offset = random::<u8>();
    let url = format!("https://openlibrary.org/subjects/{}.json?limit={}&offset={}", genre, limit, offset);
    let response = reqwest::get(&url).await;
    let response = match response {
        Ok(response) => response,
        Err(_) => {
            println!("No books found with the genre: {}", genre);
            std::process::exit(1);
        }
    };
    let text = response.text().await;
    let api_response = serde_json::from_str(&text.unwrap());
    let api_response = match api_response {
        Ok(api_response) => api_response,
        Err(_) => {
            println!("No books found with the genre: {}", genre);
            std::process::exit(1);
        }
    };
    api_response
}

fn display_books(api_response: Value) {
    let docs = api_response["docs"].as_array();
    let docs = match docs {
        Some(docs) => docs,
        None => {
            api_response["works"].as_array().unwrap()
        }
    };
    println!("Found {} books", docs.len());
    println!("{}", "-".repeat(50));
    println!("{}", "-".repeat(50));
    for doc in docs {
        let title = doc["title"].as_str();
        let title = match title {
            Some(title) => title,
            None => continue
        };
        let author = doc["author_name"].as_array();
        let author = match author {
            Some(author) => author,
            None => &{
                vec![Value::String("Unknown".to_string())]
            }
        };
        let author = author[0].as_str();
        let author = match author {
            Some(author) => author,
            None => "Unknown"
        };
        let isbn = doc["isbn"].as_array();
        let isbn = match isbn {
            Some(isbn) => isbn,
            None => &{
                vec![Value::String("Unknown".to_string())]
            }
        };
        let key = doc["key"].as_str().unwrap();
        let url = format!("https://openlibrary.org{}", key);
        let isbn = isbn[0].as_str().unwrap();
        println!("Title: {}", title);
        println!("Author: {}", author);
        println!("ISBN: {}", isbn);
        println!("URL: {}", url);
        println!("{}", "-".repeat(50));
    }
    println!("{}", "-".repeat(50));
}

fn display_subject_titles(api_response: Value) {
    let works = api_response["works"].as_array().unwrap();
    for work in works {
        let title = work["title"].as_str().unwrap();
        println!("Title: {}", title);
        println!();
    }
}

fn display_isbn_books(api_response: Value) {
    let items = api_response["items"].as_array().unwrap();
    println!("Found {} matches", items.len());
    println!("{}", "-".repeat(50));
    println!("{}", "-".repeat(50));
    for item in items {
        let url = item["itemURL"].as_str().unwrap();
        let from_record = item["fromRecord"].as_str().unwrap();
        let records = api_response["records"][from_record].clone();
        let data = records["data"].clone();
        let title = data["title"].as_str().unwrap();
        let authors = data["authors"].as_array().unwrap();
        let author_names = authors.iter().map(|author| author["name"].as_str().unwrap()).collect::<Vec<&str>>();
        let author_names = author_names.join(", ");
        let isbn = data["identifiers"]["isbn_10"].as_str();
        let isbn = match isbn {
            Some(isbn) => isbn,
            None => data["identifiers"]["isbn_13"].as_str().unwrap_or("Unknown")
        };
        println!("Title: {}", title);
        println!("Authors: {}", author_names);
        println!("ISBN: {}", isbn);
        println!("URL: {}", url);
        println!("{}", "-".repeat(50));
    }
    println!("{}", "-".repeat(50));

}

fn get_random_book_title(api_response: Value) -> String {
    let books = api_response["works"].as_array().unwrap();
    let book = books[(random::<u32>() % books.len() as u32) as usize].clone();
    let title = book["title"].as_str().unwrap();
    title.to_string()
}
