use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use sqlx::prelude::FromRow;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "server", derive(FromRow))]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub price: u32,
    pub isbn: String,
}

#[server]
pub async fn load_books(query: String) -> Result<Vec<Book>> {
    use crate::db::DB;

    let db = DB::get().await;

    let books = if query.is_empty() {
        sqlx::query_as::<_, Book>(r#"SELECT id, title, author, publisher, price, isbn FROM books"#)
            .fetch_all(db.pool())
            .await?
    } else {
        sqlx::query_as::<_, Book>(
            r#"SELECT id, title, author, publisher, price, isbn FROM books WHERE title LIKE ? OR author LIKE ?"#,
        )
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .fetch_all(db.pool())
        .await?
    };

    Ok(books)
}
