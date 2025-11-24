use dioxus::{fullstack::serde::Serialize, prelude::*};
use serde::Deserialize;

#[cfg(feature = "server")]
use crate::db::DB;

#[cfg(feature = "server")]
use sqlx::prelude::FromRow;

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(FromRow))]
struct Book {
    title: String,
    author: String,
    publisher: String,
    price: u32,
    isbn: String,
}

#[server]
async fn get_books(search_query: Option<String>) -> Result<Vec<Book>> {
    let db = DB::get().await;

    let books = if let Some(query) = search_query {
        sqlx::query_as::<_, Book>(
            r#"SELECT title, author, publisher, price, isbn FROM books WHERE title LIKE ? OR author LIKE ?"#,
        )
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .fetch_all(db.pool())
        .await?
    } else {
        sqlx::query_as::<_, Book>(r#"SELECT title, author, publisher, price, isbn FROM books"#)
            .fetch_all(db.pool())
            .await?
    };

    Ok(books)
}
