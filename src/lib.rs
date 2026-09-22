use std::error::Error;

use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};
use std::time::Duration;
use ureq;
use ureq::Agent;

fn agent() -> Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .build()
        .into()
}

pub struct WebMdprMedia {
    url: String,
}

impl WebMdprMedia {
    pub fn new(url: String) -> WebMdprMedia {
        WebMdprMedia { url }
    }

    fn get_image_index(&self, agent: &Agent) -> Result<String, Box<dyn Error>> {
        const HOST: &str = "https://mdpr.jp";
        const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36";

        let url = self.url.trim();
        if !url.contains(HOST) {
            return Ok(String::from(""));
        }

        if url.contains("photo/detail") {
            return Ok(url.to_string());
        }

        let body = agent
            .get(url)
            .header("User-Agent", USER_AGENT)
            .call()?
            .body_mut()
            .read_to_string()?;

        let document = Document::from_read(body.as_bytes())?;
        let nodes = document.find(Class("c-image__image"));

        for node in nodes {
            if let Some(href) = node.attr("href") {
                if href.contains("/photo/detail") {
                    let full = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://mdpr.jp{}", href)
                    };

                    return Ok(full);
                }
            }
        }

        Ok(String::from(""))
    }

    fn get_image_urls(
        &self,
        agent: &Agent,
        image_index: &str,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut urls: Vec<String> = vec![];

        const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/153.0.0.0 Safari/537.36";

        let body = agent
            .get(image_index)
            .header("User-Agent", USER_AGENT)
            .call()?
            .body_mut()
            .read_to_string()?;

        let document = Document::from_read(body.as_bytes())?;
        let nodes = document.find(
            Name("main")
                .and(|n: &Node| n.attr("id") == Some("js-main-content"))
                .descendant(Class("pg-photo__webImageList"))
                .descendant(Name("img")),
        );

        for node in nodes {
            if let Some(src) = node.attr("src") {
                if !src.contains("img_protect") {
                    let replaced = src.replace("/thumb/", "/");
                    let clean = replaced.split('?').next().unwrap_or(&replaced);
                    urls.push(clean.to_string());
                }
            }
        }

        Ok(urls)
    }
}

pub fn mdpr_images(url: String) -> Result<Vec<String>, Box<dyn Error>> {
    let agent = agent();

    let web = WebMdprMedia::new(url);

    let image_index = web.get_image_index(&agent)?;
    if image_index.is_empty() {
        return Ok(vec![]);
    }

    web.get_image_urls(&agent, &image_index)
}

#[test]
fn mdpr_test() {
    let url = "https://mdpr.jp/cinema/3928728";
    let web = WebMdprMedia::new(url.to_string());

    let agent = agent();
    let index = web.get_image_index(&agent).unwrap();
    println!("image index: {}", index);

    assert!(index.contains("14567030"), "image_index invalid: {}", index);

    let image_urls = web.get_image_urls(&agent, &index);
    let urls = match image_urls {
        Ok(urls) => {
            println!("image urls len: {}", urls.len());
            urls
        }
        Err(error) => {
            panic!("get_image_urls failed: {}", error);
        }
    };

    assert_eq!(urls.len(), 49, "unexpected image count: {}", urls.len());
}
