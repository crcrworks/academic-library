use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
pub struct BookFormErrors {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub price: Option<String>,
    pub isbn: Option<String>,
}

impl BookFormErrors {
    pub fn has_errors(&self) -> bool {
        self.title.is_some()
            || self.author.is_some()
            || self.publisher.is_some()
            || self.price.is_some()
            || self.isbn.is_some()
    }
}

pub fn validate_book_form(
    title: &str,
    author: &str,
    publisher: &str,
    price_str: &str,
    isbn: &str,
) -> BookFormErrors {
    let mut errors = BookFormErrors::default();

    // タイトル検証
    let title_trimmed = title.trim();
    if title_trimmed.is_empty() {
        errors.title = Some("タイトルは必須です".to_string());
    } else if title_trimmed.len() > 200 {
        errors.title = Some("タイトルは200文字以内で入力してください".to_string());
    }

    // 著者検証
    let author_trimmed = author.trim();
    if author_trimmed.is_empty() {
        errors.author = Some("著者は必須です".to_string());
    } else if author_trimmed.len() > 100 {
        errors.author = Some("著者名は100文字以内で入力してください".to_string());
    }

    // 出版社検証
    let publisher_trimmed = publisher.trim();
    if publisher_trimmed.is_empty() {
        errors.publisher = Some("出版社は必須です".to_string());
    } else if publisher_trimmed.len() > 100 {
        errors.publisher = Some("出版社名は100文字以内で入力してください".to_string());
    }

    // 価格検証
    if price_str.trim().is_empty() {
        errors.price = Some("価格は必須です".to_string());
    } else {
        match price_str.trim().parse::<u32>() {
            Ok(price) => {
                if price == 0 {
                    errors.price = Some("価格は0より大きい値を入力してください".to_string());
                } else if price > 1_000_000 {
                    errors.price = Some("価格は1,000,000円以下で入力してください".to_string());
                }
            }
            Err(_) => {
                errors.price = Some("価格は正の整数で入力してください".to_string());
            }
        }
    }

    // ISBN検証
    let isbn_trimmed = isbn.trim();
    if isbn_trimmed.is_empty() {
        errors.isbn = Some("ISBNは必須です".to_string());
    } else if !is_valid_isbn_format(isbn_trimmed) {
        errors.isbn =
            Some("ISBNは10桁または13桁の数字で入力してください（ハイフンあり可）".to_string());
    }

    errors
}

fn is_valid_isbn_format(isbn: &str) -> bool {
    // ハイフンを除去して数字のみを抽出
    let digits: String = isbn.chars().filter(|c| c.is_ascii_digit()).collect();
    // ISBN-10（10桁）またはISBN-13（13桁）
    digits.len() == 10 || digits.len() == 13
}
