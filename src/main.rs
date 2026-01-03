use std::time::Duration;

use dioxus::prelude::*;
use dioxus_sdk::time::use_debounce;
use gloo_timers::future::sleep;

#[cfg(feature = "server")]
mod db;

mod books;
mod validation;

use crate::books::{create_book, load_books, Book, CreateBookForm};
use crate::validation::{validate_book_form, BookFormErrors};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
	#[route("/?:query")]
	Home {query: String},
	#[route("/create")]
	CreateBook {},
}

#[derive(Clone, PartialEq)]
enum SubmitState {
	Idle,
	Submitting,
	Success(Book),
	Error(String),
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home(query: String) -> Element {
    let mut query_signal = use_signal(|| query.clone());

    let books_resource = use_resource(move || {
        let query = query_signal();
        async move { load_books(query).await }
    });

    let on_query_change = move |new_query: String| {
        query_signal.set(new_query);
    };

    let content = match books_resource.read().as_ref() {
        Some(Ok(books)) => {
            rsx! {
                Books { books: books.clone() }
            }
        }
        Some(Err(e)) => {
            eprintln!("{e}");
            rsx! {
                div { "Error: {e}" }
            }
        }
        None => rsx! {
            div { "loading..." }
        },
    };

	rsx! {
		div { class: "container mx-auto p-6",
			div { class: "flex flex-col md:flex-row justify-between items-start md:items-center gap-4 mb-8",
				h1 { class: "text-3xl font-bold text-gray-800", "書籍検索システム" }
				Link {
					to: Route::CreateBook {},
					class: "px-6 py-3 bg-blue-600 text-white font-semibold rounded-md hover:bg-blue-700 transition-colors shadow-md",
					"+ 新しい書籍を追加"
				}
			}

			div { class: "mb-6",
				SearchInput { query: query.clone(), on_query_change }
			}

			div { class: "flex flex-col", {content} }
		}
	}
}

#[component]
fn Books(books: Vec<Book>) -> Element {
    if books.is_empty() {
        rsx! { "No books to show" }
    } else {
        rsx! {
            div { class: "flex flex-wrap gap-4",
                for book in books {
                    div { class: "min-w-80 border rounded-lg p-4 shadow hover:shadow-lg transition-shadow",
                        p { class: "text-sm text-gray-500", "ID: {book.id}" }
                        p { class: "font-bold text-lg mt-2", {book.title} }
                        p { class: "text-gray-700 mt-1", {book.author} }
                        p { class: "text-gray-600 text-sm mt-1", {book.publisher} }
                        p { class: "text-green-600 font-semibold mt-2", "¥{book.price}" }
                        p { class: "text-xs text-gray-400 mt-1", "ISBN: {book.isbn}" }
                    }
                }
            }
        }
    }
}

#[component]
fn SearchInput(query: String, on_query_change: EventHandler<String>) -> Element {
	let navigator = use_navigator();
	let mut search_query = use_signal(|| query.clone());

	let mut debounce = use_debounce(Duration::from_secs(1), move |query: String| {
		search_query.set(query.clone());
		on_query_change.call(query.clone());
		navigator.push(Route::Home { query });
	});

	let oninput = move |event: Event<FormData>| {
		let new_query = event.value();
		debounce.action(new_query);
	};

	rsx! {
		input {
			class: "border-1",
			value: "{search_query}",
			oninput
		}
	}
}

#[component]
fn FormField(label: String, required: bool, error: Option<String>, children: Element) -> Element {
	rsx! {
		div { class: "form-group",
			label { class: "block text-sm font-semibold text-gray-700 mb-2",
				{label}
				if required {
					span { class: "text-red-500 ml-1", "*" }
				}
			}
			{children}
			if let Some(err_msg) = error {
				p { class: "text-red-500 text-sm mt-1 flex items-center gap-1",
					span { "⚠" }
					{err_msg}
				}
			}
		}
	}
}

#[component]
fn CreateBook() -> Element {
	let navigator = use_navigator();

	let mut title = use_signal(String::new);
	let mut author = use_signal(String::new);
	let mut publisher = use_signal(String::new);
	let mut price_str = use_signal(String::new);
	let mut isbn = use_signal(String::new);

	let mut validation_errors = use_signal(BookFormErrors::default);
	let mut submit_state = use_signal(|| SubmitState::Idle);

	let handle_submit = move |event: Event<FormData>| {
		event.prevent_default();

		spawn(async move {
			let errors = validate_book_form(
				&title(),
				&author(),
				&publisher(),
				&price_str(),
				&isbn(),
			);

			validation_errors.set(errors.clone());

			if errors.has_errors() {
				return;
			}

			let price = match price_str().trim().parse::<u32>() {
				Ok(p) => p,
				Err(_) => {
					submit_state.set(SubmitState::Error("価格の形式が無効です".to_string()));
					return;
				}
			};

			submit_state.set(SubmitState::Submitting);

			let form_data = CreateBookForm {
				title: title(),
				author: author(),
				publisher: publisher(),
				price,
				isbn: isbn(),
			};

			match create_book(form_data).await {
				Ok(book) => {
					submit_state.set(SubmitState::Success(book));

					spawn(async move {
						sleep(Duration::from_secs(2)).await;
						navigator.push(Route::Home {
							query: String::new(),
						});
					});
				}
				Err(e) => {
					submit_state.set(SubmitState::Error(format!("登録に失敗しました: {}", e)));
				}
			}
		});
	};

	rsx! {
		div { class: "container mx-auto p-6 max-w-2xl",
			div { class: "flex justify-between items-center mb-6",
				h1 { class: "text-2xl font-bold", "書籍の新規登録" }
				Link {
					to: Route::Home { query: String::new() },
					class: "text-blue-600 hover:underline",
					"← 一覧に戻る"
				}
			}

			{match submit_state() {
				SubmitState::Success(ref book) => rsx! {
					div { class: "bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded mb-4",
						p { class: "font-semibold", "✓ 書籍を登録しました" }
						p { class: "text-sm mt-1", "「{book.title}」を登録しました。一覧ページに戻ります..." }
					}
				},
				SubmitState::Error(ref err) => rsx! {
					div { class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4",
						p { class: "font-semibold", "✗ エラーが発生しました" }
						p { class: "text-sm mt-1", "{err}" }
					}
				},
				_ => rsx! {}
			}}

			form {
				onsubmit: handle_submit,
				class: "space-y-6 bg-white shadow-md rounded px-8 pt-6 pb-8",

				FormField {
					label: "タイトル",
					required: true,
					error: validation_errors().title,
					input {
						r#type: "text",
						class: "w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900",
						class: if validation_errors().title.is_some() { " border-red-500" } else { " border-gray-300" },
						value: "{title}",
						placeholder: "例: プログラミング言語Rust",
						maxlength: "200",
						oninput: move |event: Event<FormData>| {
							title.set(event.value());
							validation_errors.with_mut(|e| e.title = None);
						}
					}
				}

				FormField {
					label: "著者",
					required: true,
					error: validation_errors().author,
					input {
						r#type: "text",
						class: "w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900",
						class: if validation_errors().author.is_some() { " border-red-500" } else { " border-gray-300" },
						value: "{author}",
						placeholder: "例: Steve Klabnik, Carol Nichols",
						maxlength: "100",
						oninput: move |event: Event<FormData>| {
							author.set(event.value());
							validation_errors.with_mut(|e| e.author = None);
						}
					}
				}

				FormField {
					label: "出版社",
					required: true,
					error: validation_errors().publisher,
					input {
						r#type: "text",
						class: "w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900",
						class: if validation_errors().publisher.is_some() { " border-red-500" } else { " border-gray-300" },
						value: "{publisher}",
						placeholder: "例: 技術評論社",
						maxlength: "100",
						oninput: move |event: Event<FormData>| {
							publisher.set(event.value());
							validation_errors.with_mut(|e| e.publisher = None);
						}
					}
				}

				FormField {
					label: "価格（円）",
					required: true,
					error: validation_errors().price,
					input {
						r#type: "number",
						class: "w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900",
						class: if validation_errors().price.is_some() { " border-red-500" } else { " border-gray-300" },
						value: "{price_str}",
						placeholder: "例: 3520",
						min: "1",
						max: "1000000",
						step: "1",
						oninput: move |event: Event<FormData>| {
							price_str.set(event.value());
							validation_errors.with_mut(|e| e.price = None);
						}
					}
				}

				FormField {
					label: "ISBN",
					required: true,
					error: validation_errors().isbn,
					input {
						r#type: "text",
						class: "w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900",
						class: if validation_errors().isbn.is_some() { " border-red-500" } else { " border-gray-300" },
						value: "{isbn}",
						placeholder: "例: 978-4798158228",
						oninput: move |event: Event<FormData>| {
							isbn.set(event.value());
							validation_errors.with_mut(|e| e.isbn = None);
						}
					}
					p { class: "text-xs text-gray-500 mt-1",
						"10桁または13桁の数字を入力してください（ハイフンあり可）"
					}
				}

				div { class: "flex gap-4 pt-4",
					button {
						r#type: "submit",
						class: "flex-1 px-6 py-3 bg-blue-600 text-white font-semibold rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors",
						disabled: matches!(submit_state(), SubmitState::Submitting | SubmitState::Success(_)),
						{match submit_state() {
							SubmitState::Submitting => "登録中...",
							SubmitState::Success(_) => "登録完了",
							_ => "登録する",
						}}
					}
					Link {
						to: Route::Home { query: String::new() },
						class: "flex-1 px-6 py-3 border border-gray-300 text-center rounded-md hover:bg-gray-50 transition-colors",
						"キャンセル"
					}
				}
			}
		}
	}
}
