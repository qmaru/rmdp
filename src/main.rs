use std::env;

use rmdp::mdpr_images;

fn main() {
    let user_url = env::args().nth(1);
    let user_format = env::args().nth(2);

    let mut format = String::new();
    if let Some(uformat) = user_format {
        if uformat != "json" {
            format = String::from("text");
        }
    }

    if let Some(url) = user_url {
        let url = String::from(url);
        let image_urls = mdpr_images(url);
        match image_urls {
            Ok(urls) => {
                if format == "text" {
                    for url in urls {
                        println!("{}", url)
                    }
                } else {
                    println!("{:?}", urls)
                }
            }
            Err(error) => println!("error: {}", error),
        }
    } else {
        let prog = env::args().nth(0).expect("error: bad input");
        println!("Usage: {} <url> [text|json]", prog)
    }
}
