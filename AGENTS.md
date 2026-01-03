# Agent Guidelines for Academic Library

This is a Dioxus 0.7 fullstack web application for searching and displaying books from a SQLite database.

## Build, Test, and Development Commands

### Development Server
```bash
dx serve                    # Start dev server (default: web platform)
dx serve --platform web     # Explicitly serve web platform
dx serve --platform desktop # Serve desktop platform
```

### Rust Commands
```bash
cargo check                 # Check code for errors
cargo build                 # Build the project
cargo build --release       # Build optimized release
cargo fmt                   # Format code with rustfmt
cargo clippy               # Run linter
cargo test                 # Run all tests
cargo run                  # Run without hot-reloading
```

### Dioxus CLI Commands
```bash
dx build                   # Build project and assets
dx bundle                  # Bundle into shippable object
dx check                   # Check project for issues
dx fmt                     # Format RSX code
```

### Database Commands (using Task)
```bash
task db:setup              # Initial setup (create + migrate)
task db:create             # Create database
task db:migrate            # Run migrations
task db:migrate:add -- <name>  # Add new migration
task db:migrate:revert     # Revert last migration
task db:migrate:info       # Check migration status
task db:check              # Check DB connection and schema
task db:drop               # Drop database (interactive)
```

## Code Style Guidelines

### Formatting
- **Indentation**: Use TABS (not spaces) - project uses tab indentation consistently
- **Line length**: Keep reasonable (rustfmt defaults)
- **RSX formatting**: Use `dx fmt` for RSX code

### Imports
- Group imports: `std` → external crates → `dioxus::prelude::*` → local modules
- Use `use dioxus::prelude::*;` as the standard Dioxus import
- Feature-gate imports when needed: `#[cfg(feature = "server")]`

```rust
use std::time::Duration;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use sqlx::prelude::FromRow;

mod db;
mod books;
```

### Naming Conventions
- **Components**: PascalCase, must start with capital letter or contain underscore
- **Functions**: snake_case
- **Constants**: SCREAMING_SNAKE_CASE for assets (`const FAVICON: Asset = asset!("/assets/favicon.ico")`)
- **Types/Structs**: PascalCase
- **Database tables**: lowercase snake_case

### Type Annotations
- Always derive `PartialEq` and `Clone` for component props
- Use `#[cfg_attr(feature = "server", derive(FromRow))]` for database models
- Prefer explicit types for clarity in function signatures

```rust
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "server", derive(FromRow))]
pub struct Book {
	pub id: i32,
	pub title: String,
	// ...
}
```

### Error Handling
- Use `Result<T, E>` for fallible operations
- Use `?` operator for error propagation
- Use `expect()` with descriptive messages for initialization/setup code
- Server functions return `Result<T>` (Dioxus handles the error type)

```rust
#[server]
pub async fn load_books(query: String) -> Result<Vec<Book>> {
	let db = DB::get().await;
	let books = sqlx::query_as::<_, Book>("SELECT ...")
		.fetch_all(db.pool())
		.await?;  // Use ? for propagation
	Ok(books)
}
```

### Async/Await
- Use `async fn` for asynchronous operations
- Use `tokio` runtime (already configured)
- Use `use_resource` hook for async data loading in components
- Pattern match on resource state: `Some(Ok(data))`, `Some(Err(e))`, `None`

## Dioxus 0.7 Specific Patterns

### CRITICAL: Dioxus 0.7 Breaking Changes
- **NO `cx` parameter** - removed in 0.7
- **NO `Scope`** - removed in 0.7
- **NO `use_state`** - use `use_signal` instead
- All documentation and examples MUST use Dioxus 0.7 APIs only

### Components
```rust
#[component]
fn MyComponent(name: String, count: ReadOnlySignal<i32>) -> Element {
	rsx! {
		div { "Hello {name}, count: {count}" }
	}
}
```

- Annotate with `#[component]` macro
- Props must be owned values (use `String`, not `&str`)
- Props must implement `PartialEq` and `Clone`
- Use `ReadOnlySignal<T>` for reactive props

### State Management
```rust
// Local state
let mut count = use_signal(|| 0);
count();           // Read (clone value)
count.read();      // Read (borrow)
*count.write() = 5;  // Write
count.with_mut(|c| *c += 1);  // Mutate with closure

// Memoized values
let doubled = use_memo(move || count() * 2);

// Async resources
let books = use_resource(move || async move {
	load_books(query()).await
});
```

### RSX Syntax
```rust
rsx! {
	div { class: "container",
		// Prefer for loops over iterators
		for book in books {
			p { "{book.title}" }
		}
		
		// Conditionals
		if condition {
			div { "True branch" }
		}
		
		// Expressions in braces
		{some_expression}
	}
}
```

### Assets
```rust
const FAVICON: Asset = asset!("/assets/favicon.ico");

rsx! {
	document::Link { rel: "icon", href: FAVICON }
	document::Stylesheet { href: asset!("/assets/main.css") }
}
```

## Fullstack Features

### Server Functions
Use `#[server]` macro to define backend-only functions:

```rust
#[server]
pub async fn load_books(query: String) -> Result<Vec<Book>> {
	// This code ONLY runs on server
	let db = DB::get().await;
	// ... database queries
	Ok(books)
}
```

- Automatically creates API endpoint on server
- Generates client-side function that makes HTTP request
- Use for database access, API calls, authentication

### Feature Flags
```toml
[features]
default = ["web"]
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
mobile = ["dioxus/mobile"]
server = ["dep:dotenvy", "dep:sqlx", "dioxus/server"]
```

Use `#[cfg(feature = "server")]` to conditionally compile server-only code:
```rust
#[cfg(feature = "server")]
mod db;  // Only compiled with server feature
```

### Routing
```rust
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
	#[route("/?:query")]
	Home { query: String },
}

// In component
rsx! { Router::<Route> {} }
```

## Project-Specific Patterns

### Database Access
- Use singleton pattern with `OnceCell` for DB connection pool
- Always use parameterized queries (never string concatenation)
- Use `sqlx::query_as` with type annotations

```rust
sqlx::query_as::<_, Book>(
	"SELECT id, title, author, publisher, price, isbn FROM books WHERE title LIKE ?"
)
.bind(format!("%{}%", query))
.fetch_all(db.pool())
.await?
```

### Environment Variables
- Store in `.env` file (gitignored, see `.env.example`)
- Load with `dotenvy::dotenv()` on server startup
- Required: `DATABASE_URL=sqlite:db/books.db`

### Component Organization
- Main app in `src/main.rs`
- Domain models in separate files (`src/books.rs`, `src/db.rs`)
- Feature-gate server-side code appropriately

## Common Pitfalls

1. **Don't use old Dioxus 0.5/0.6 patterns** - no `cx`, `Scope`, or `use_state`
2. **Don't forget feature gates** - server code must be behind `#[cfg(feature = "server")]`
3. **Don't use `&str` in props** - use `String` (owned values only)
4. **Don't forget to derive PartialEq and Clone** on prop types
5. **Don't use tabs and spaces mixed** - this project uses TABS
6. **Don't concatenate SQL strings** - always use parameterized queries
