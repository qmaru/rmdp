use std::env;

use rmdp::MdprMedia;

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
        let mdpr = MdprMedia::new(url);
        let image_index = mdpr.get_image_index();
        match image_index {
            Ok(index) => {
                let image_urls = mdpr.get_image_urls(index);
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
            }
            Err(error) => println!("error: {}", error),
        }
    } else {
        let prog = env::args().nth(0).expect("error: bad input");
        println!("Usage: {} <url> [text|json]", prog)
    }
}
