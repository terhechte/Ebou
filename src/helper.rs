pub fn format_number(number: i64) -> String {
    use numfmt::*;
    let mut formatter = Formatter::default()
        .scales(Scales::metric())
        .precision(Precision::Decimals(0));
    formatter.fmt(number as f64).to_string()
}

/// Try to parse a mastodon url such as
/// https://mstdn.social/@briannawu
/// into a mastodon id such as
/// briannawu@mstdn.social
pub fn parse_user_url(url: &str) -> Option<String> {
    use url::Url;
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host()?;
    let user = parsed.path_segments()?.next()?;
    if !user.starts_with('@') {
        return None;
    }
    let username = &user[1..];
    Some(format!("{username}@{host}"))
}

mod clean_html_content {
    use html5gum::{HtmlString, Token, Tokenizer};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub enum HtmlItem {
        Mention { url: String, name: String },
        Hashtag { name: String },
        Link { url: String, name: String },
        Text { content: String },
        Image { url: String },
        Break,
    }

    pub fn clean_html(html: &str) -> (String, Vec<HtmlItem>) {
        let mut text = String::new();

        let mut collected = Vec::new();

        let mut last_text = String::new();
        let mut last_url: Option<String> = None;
        let mut is_href = false;

        let attr_href = HtmlString("href".as_bytes().to_owned());
        let attr_src = HtmlString("src".as_bytes().to_owned());

        for token in Tokenizer::new(html).infallible() {
            match token {
                Token::StartTag(tag) => {
                    let name = std::str::from_utf8(&tag.name.0);
                    match name {
                        Ok(a) if a == "a" => {
                            is_href = true;
                            last_text.clear();
                            if let Some(href) = tag
                                .attributes
                                .get(&attr_href)
                                .and_then(|e| std::str::from_utf8(&e.0).ok())
                            {
                                last_url = Some(href.to_string());
                            }
                        }
                        Ok(a) if a == "br" => {
                            collected.push(HtmlItem::Break);
                            text.push(' ');
                            last_text.push(' ');
                        }
                        Ok(img) if img == "img" => {
                            if let Some(url) = tag
                                .attributes
                                .get(&attr_src)
                                .and_then(|e| std::str::from_utf8(&e.0).ok())
                            {
                                collected.push(HtmlItem::Image {
                                    url: url.to_string(),
                                });
                            }
                            text.push(' ');
                            last_text.push(' ');
                        }
                        _ => (),
                    }
                }
                Token::EndTag(tag) => {
                    let name = std::str::from_utf8(&tag.name.0);
                    match name {
                        Ok(a) if a == "a" => {
                            let Some(url) = last_url.take() else { continue };
                            let name = last_text.clone();
                            match last_text.chars().next() {
                                Some('@') => collected.push(HtmlItem::Mention { url, name }),
                                Some('#') => collected.push(HtmlItem::Hashtag { name }),
                                _ => collected.push(HtmlItem::Link { url, name }),
                            }

                            last_text.clear();
                            is_href = false;
                        }
                        Ok(a) if a == "p" => {
                            collected.push(HtmlItem::Break);
                            collected.push(HtmlItem::Break);
                            text.push(' ');
                            last_text.push(' ');
                        }
                        _ => (),
                    }
                }
                Token::String(s) => {
                    let sx = std::str::from_utf8(&s.0).unwrap_or_default();
                    text.push_str(sx);
                    last_text.push_str(sx);
                    if !is_href {
                        collected.push(HtmlItem::Text {
                            content: sx.to_string(),
                        })
                    }
                }
                Token::Comment(_) => (),
                Token::Doctype(_) => (),
                Token::Error(_) => (),
            }
        }

        // if we have > 0 brs at the end, remove them
        let mut brs = 0;
        for n in collected.iter().rev() {
            if matches!(n, HtmlItem::Break) {
                brs += 1;
            } else {
                break;
            }
        }
        for _ in 0..brs {
            collected.remove(collected.len() - 1);
        }

        (text, collected)
    }
}

pub use clean_html_content::{clean_html, HtmlItem};
