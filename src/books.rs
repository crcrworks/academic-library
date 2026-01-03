use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use sqlx::prelude::FromRow;

#[cfg(feature = "server")]
use dioxus::CapturedError;

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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CreateBookForm {
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

#[server]
pub async fn create_book(form_data: CreateBookForm) -> Result<Book> {
	use crate::db::DB;

	if form_data.title.trim().is_empty() {
		return Err(ServerFnError::new("タイトルは必須です").into());
	}
	if form_data.author.trim().is_empty() {
		return Err(ServerFnError::new("著者は必須です").into());
	}
	if form_data.publisher.trim().is_empty() {
		return Err(ServerFnError::new("出版社は必須です").into());
	}
	if form_data.isbn.trim().is_empty() {
		return Err(ServerFnError::new("ISBNは必須です").into());
	}

	let isbn_digits: String = form_data
		.isbn
		.chars()
		.filter(|c| c.is_ascii_digit())
		.collect();
	if isbn_digits.len() != 10 && isbn_digits.len() != 13 {
		return Err(ServerFnError::new(
			"ISBNは10桁または13桁の数字である必要があります",
		)
		.into());
	}

	let db = DB::get().await;

	let book = sqlx::query_as::<_, Book>(
		r#"INSERT INTO books (title, author, publisher, price, isbn) 
		   VALUES (?, ?, ?, ?, ?) 
		   RETURNING id, title, author, publisher, price, isbn"#,
	)
	.bind(form_data.title.trim())
	.bind(form_data.author.trim())
	.bind(form_data.publisher.trim())
	.bind(form_data.price)
	.bind(form_data.isbn.trim())
	.fetch_one(db.pool())
	.await
	.map_err(|e| {
		let err: CapturedError = ServerFnError::new(format!("データベースエラー: {}", e)).into();
		err
	})?;

	Ok(book)
}
