use pulldown_cmark::{self, Event, Tag};
use std::fs;
use std::path::PathBuf;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    for path in html_and_text_files() {
        fs::remove_file(path).unwrap();
    }

    for path in markdown_files() {
        let markdown = fs::read_to_string(&path).unwrap();
        let (html, text_parts) = markdown_to_html_and_text_parts(&markdown);

        let html_path = format!("generated_guides/{}.html", path.file_stem().unwrap().to_str().unwrap());
        fs::write(html_path, html).unwrap();

        let text_path = format!("generated_guides/{}.txt", path.file_stem().unwrap().to_str().unwrap());
        fs::write(text_path, text_parts.join(" ")).unwrap();
    }
}

fn html_and_text_files() -> Vec<PathBuf> {
    fs::read_dir("generated_guides")
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();

            match path.extension().unwrap_or_default().to_str().unwrap() {
                "html" | "txt" => Some(path),
                _ => None,
            }
        })
        .collect()
}

fn markdown_files() -> Vec<PathBuf> {
    fs::read_dir("guides")
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.extension().unwrap_or_default() == "md" {
                Some(path)
            } else {
                None
            }
        })
        .collect()
}

fn markdown_to_html_and_text_parts(markdown: &str) -> (String, Vec<String>) {
    let parser = pulldown_cmark::Parser::new(markdown);

    let mut html = String::new();
    let text_parts = Rc::new(RefCell::new(Vec::<String>::new()));

    let parser = transform_code_blocks(parser);
    let parser = extract_text(parser, text_parts.clone());

    pulldown_cmark::html::push_html(&mut html, parser);
    (html, text_parts.replace(Vec::new()))
}

fn extract_text<'a, I>(parser: I, text_parts: Rc<RefCell<Vec<String>>>) -> impl Iterator<Item = Event<'a>>
    where
        I: Iterator<Item = Event<'a>>,
{
    parser.map(move |event| {
        match event {
            Event::Text(text) => {
                text_parts.borrow_mut().push(text.to_string());
                Event::Text(text)
            },
            Event::Code(code) => {
                text_parts.borrow_mut().push(code.to_string());
                Event::Code(code)
            },
            Event::FootnoteReference(reference) => {
                text_parts.borrow_mut().push(reference.to_string());
                Event::FootnoteReference(reference)
            },
            _ => event
        }
    })
}

fn transform_code_blocks<'a, I>(parser: I) -> impl Iterator<Item = Event<'a>>
    where
        I: Iterator<Item = Event<'a>>,
{
    parser.map(|event| {
        match event {
            Event::Start(Tag::CodeBlock(code_lang)) => {
                let lang = if code_lang.is_empty() {
                    String::new()
                } else {
                    format!(" lang=\"{}\"", code_lang)
                };
                Event::Html(format!("<code-block{}>", lang).into())
            }
            Event::End(Tag::CodeBlock(_)) => {
                Event::Html("</code-block>".into())
            }
            _ => event
        }
    })
}

