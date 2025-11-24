use dioxus::prelude::*;

#[cfg(feature = "server")]
mod db;

mod books;

use crate::books::{load_books, Book};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/?:query")]
    Home {query: String},
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
        div { class: "flex flex-col gap-10",
            div { SearchInput { query: query.clone(), on_query_change } }
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

    let oninput = move |event: Event<FormData>| {
        let new_query = event.value();
        search_query.set(new_query.clone());
        on_query_change.call(new_query.clone());
        navigator.push(Route::Home { query: new_query });
    };

    rsx! {
        input {
            class: "border-1",
            value: "{search_query}",
            oninput
        }
    }
}
