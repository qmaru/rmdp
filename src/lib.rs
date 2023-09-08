use std::error::Error;

use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use serde_json::Value;
use ureq;
use urlencoding::decode;

pub struct MdprMedia {
    url: String,
}

impl MdprMedia {
    pub fn new(url: String) -> MdprMedia {
        MdprMedia { url }
    }

    fn get_article_id(&self) -> &str {
        let mut article_id = "";
        let url = self.url.trim();
        if url.contains("https://mdpr.jp") {
            if !url.contains("photo/details") {
                let url_parts: Vec<&str> = url.split("/").collect();
                let url_len = url_parts.len();
                article_id = url_parts[url_len - 1];
            }
        }
        article_id
    }

    pub fn get_image_index(&self) -> Result<String, Box<dyn Error>> {
        let mut image_index = String::from("");

        const MDPR_HOST: &str = "https://app2-mdpr.freetls.fastly.net";
        const USER_AGENT:&str = "Mozilla/5.0 (Linux; Android 7.1.1; E6533 Build/32.4.A.1.54; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/94.0.4606.85 Mobile Safari/537.36";
        const X_REQUESTED_WITH: &str = "jp.mdpr.mdprviewer";

        let aid = self.get_article_id();
        if aid == "" {
            return Err("url is error".into());
        }

        let mobile_index = format!("{MDPR_HOST}/articles/detail/{aid}");
        let body = ureq::get(&mobile_index)
            .set("User-Agent", USER_AGENT)
            .set("X-Requested-With", X_REQUESTED_WITH)
            .call()?
            .into_string()?;

        let document = Document::from_read(body.as_bytes())?;
        let nodes = document.find(Class("p-articleBody").descendant(Name("a")));
        for node in nodes {
            let mdpr_data = node.attr("data-mdprapp-option");
            if let Some(mdpr_data) = mdpr_data {
                let mdpr_json_str = decode(mdpr_data).expect("UTF-8").into_owned();
                let mdpr_json_data: Value = serde_json::from_str(&mdpr_json_str)?;
                let mdpr_image_index = mdpr_json_data["url"].as_str();
                if let Some(mdpr_image_index) = mdpr_image_index {
                    if mdpr_image_index.contains(aid) {
                        image_index = format!("{MDPR_HOST}{mdpr_image_index}");
                    }
                } else {
                    return Err("image not found".into());
                }
            } else {
                return Err("data not found".into());
            }
        }
        Ok(image_index)
    }

    pub fn get_image_urls(&self, image_index: String) -> Result<Vec<String>, Box<dyn Error>> {
        let mut urls: Vec<String> = vec![];

        const USER_AGENT: &str = "okhttp/4.9.1";
        const MDPR_USER_AGENT: &str = "sony; E653325; android; 7.1.1; 3.10.4838(66);";

        let body = ureq::get(&image_index)
            .set("User-Agent", USER_AGENT)
            .set("mdpr-user-agent", MDPR_USER_AGENT)
            .call()?
            .into_string()?;

        let mdpr_image_data: Value = serde_json::from_str(&body)?;
        let mdpr_image_list = mdpr_image_data["list"].as_array();
        if let Some(mdpr_image_list) = mdpr_image_list {
            for image in mdpr_image_list {
                let img_url = image["url"].as_str();
                if let Some(img_url) = img_url {
                    let u = img_url.to_string();
                    urls.push(u);
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

/** `mdpr_images` 获取 mdpr 图片地址
```
use rmdp::MdprMedia;
let url = String::from("https://mdpr.jp/cinema/3928728");
let mdpr = MdprMedia::new(url);
let image_index = mdpr.get_image_index().unwrap();
let image_urls = mdpr.get_image_urls(image_index).unwrap();
assert_ne!(vec![String::from("")], image_urls);
```
 */
pub fn mdpr_images(url: String) -> Result<Vec<String>, Box<dyn Error>> {
    let url = String::from(url);
    let mdpr = MdprMedia::new(url);
    let image_index = mdpr.get_image_index()?;
    let image_urls = mdpr.get_image_urls(image_index)?;
    Ok(image_urls)
}
