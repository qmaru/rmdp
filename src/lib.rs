use std::error::Error;

use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};
use serde_json::Value;
use ureq;
use urlencoding::decode;

pub struct MobileMdprMedia {
    url: String,
}

impl MobileMdprMedia {
    pub fn new(url: String) -> MobileMdprMedia {
        MobileMdprMedia { url }
    }

    fn get_article_id(&self) -> String {
        let url = self.url.trim();

        if url.contains("https://mdpr.jp") && !url.contains("photo/detail") {
            let url_parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
            if let Some(article_id) = url_parts.last() {
                return article_id.to_string();
            }
        }

        String::from("")
    }

    pub fn get_image_index(&self) -> Result<String, Box<dyn Error>> {
        const MDPR_HOST: &str = "https://app2-mdpr.freetls.fastly.net";
        const USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 7.1.1; E6533 Build/32.4.A.1.54; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/94.0.4606.85 Mobile Safari/537.36";
        const X_REQUESTED_WITH: &str = "jp.mdpr.mdprviewer";

        let aid = self.get_article_id();
        if aid == "" {
            return Ok(String::from(""));
        }

        let mobile_index = format!("{MDPR_HOST}/articles/detail/{aid}");
        let body = ureq::get(&mobile_index)
            .header("User-Agent", USER_AGENT)
            .header("X-Requested-With", X_REQUESTED_WITH)
            .call()?
            .body_mut()
            .read_to_string()?;

        let document = Document::from_read(body.as_bytes())?;
        let nodes = document.find(Class("p-articleBody").descendant(Name("a")));

        for node in nodes {
            let mdpr_data = node.attr("data-mdprapp-option");
            if let Some(mdpr_data) = mdpr_data {
                let mdpr_json_str = decode(mdpr_data).expect("UTF-8").into_owned();
                let mdpr_json_data: Value = serde_json::from_str(&mdpr_json_str)?;
                let mdpr_image_index = mdpr_json_data["url"].as_str();

                if let Some(mdpr_image_index) = mdpr_image_index {
                    if mdpr_image_index.contains(&aid) {
                        let image_index = format!("{MDPR_HOST}{mdpr_image_index}");
                        return Ok(image_index);
                    }
                } else {
                    return Err("image not found".into());
                }
            }
        }

        Ok(String::from(""))
    }

    pub fn get_image_urls(&self, image_index: String) -> Result<Vec<String>, Box<dyn Error>> {
        let mut urls: Vec<String> = vec![];

        const USER_AGENT: &str = "okhttp/4.9.1";
        const MDPR_USER_AGENT: &str = "sony; E653325; android; 7.1.1; 3.10.4838(66);";

        let body = ureq::get(&image_index)
            .header("User-Agent", USER_AGENT)
            .header("mdpr-user-agent", MDPR_USER_AGENT)
            .call()?
            .body_mut()
            .read_to_string()?;

        let mdpr_image_data: Value = serde_json::from_str(&body)?;
        let mdpr_image_list = mdpr_image_data["list"].as_array();
        if let Some(mdpr_image_list) = mdpr_image_list {
            for image in mdpr_image_list {
                let img_url = image["url"].as_str();
                if let Some(img_url) = img_url {
                    urls.push(img_url.to_string());
                } else {
                    return Err("image not found".into());
                }
            }
        } else {
            return Err("image data error".into());
        }

        Ok(urls)
    }
}

pub struct WebMdprMedia {
    url: String,
}

impl WebMdprMedia {
    pub fn new(url: String) -> WebMdprMedia {
        WebMdprMedia { url }
    }

    fn get_image_index(&self) -> Result<String, Box<dyn Error>> {
        const HOST: &str = "https://mdpr.jp";
        const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36";

        let url = self.url.trim();
        if !url.contains(HOST) {
            return Ok(String::from(""));
        }

        if url.contains("photo/detail") {
            return Ok(url.to_string());
        }

        let body = ureq::get(url)
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

    fn get_image_urls(&self, image_index: String) -> Result<Vec<String>, Box<dyn Error>> {
        let mut urls: Vec<String> = vec![];

        const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36";

        let body = ureq::get(&image_index)
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
    let mobile = MobileMdprMedia::new(url.clone());

    let mobile_urls = match mobile.get_image_index() {
        Ok(idx) if !idx.is_empty() => match mobile.get_image_urls(idx) {
            Ok(urls) if !urls.is_empty() => Some(urls),
            Ok(_) => None,
            Err(_) => None,
        },
        Ok(_) => None,
        Err(_) => None,
    };

    if let Some(urls) = mobile_urls {
        return Ok(urls);
    }

    let web = WebMdprMedia::new(url);

    let image_index = web.get_image_index()?;
    if image_index.is_empty() {
        return Ok(vec![]);
    }

    let image_urls = web.get_image_urls(image_index)?;
    Ok(image_urls)
}

#[test]
fn mdpr_test() {
    let url = "https://mdpr.jp/cinema/3928728";
    let web = WebMdprMedia::new(url.to_string());

    let image_index = web.get_image_index();
    let index = match image_index {
        Ok(index) => {
            println!("image index: {}", index);
            index
        }
        Err(error) => {
            panic!("get_image_index failed: {}", error);
        }
    };

    assert!(index.contains("14567030"), "image_index invalid: {}", index);

    let image_urls = web.get_image_urls(index);
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
