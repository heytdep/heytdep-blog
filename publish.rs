use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::str;
use std::path::PathBuf;

fn extract_name_from_filename(filename: &str) -> &str {
    // Split the filename by "--" and take the second part
    let parts: Vec<&str> = filename.split("--").collect();
    
    parts[1]
}

fn extract_numeric_prefix(filename: &str) -> Option<i32> {
    let parts: Vec<&str> = filename.split("--").collect();
    if let Some(first_part) = parts.get(0) {
        let numeric_part = first_part.trim_matches(|c: char| !c.is_numeric());
        numeric_part.parse::<i32>().ok()
    } else {
        None
    }
}

fn get_drafts() -> Vec<PathBuf> {
    let mut paths = fs::read_dir("./drafts")
        .expect("Failed to read directory")
        .map(|entry| entry.expect("Failed to read entry").path())
        .collect::<Vec<PathBuf>>();

    // Sort the paths based on the numeric prefix in the filenames
    paths.sort_by(|a, b| {
        let a_filename = a.file_name().unwrap().to_str().unwrap();
        let b_filename = b.file_name().unwrap().to_str().unwrap();

        if let (Some(a_num), Some(b_num)) =
            (extract_numeric_prefix(a_filename), extract_numeric_prefix(b_filename))
        {
            a_num.cmp(&b_num)
        } else {
            // Fallback to comparing full filenames if numeric prefix extraction fails
            a_filename.cmp(b_filename)
        }
    });

    paths
}

fn get_comp() -> Vec<PathBuf> {
    let mut paths = fs::read_dir("./post")
        .expect("Failed to read directory")
        .map(|entry| entry.expect("Failed to read entry").path())
        .collect::<Vec<PathBuf>>();

    
    paths.sort_by(|a, b| {
        let a_filename = a.file_name().unwrap().to_str().unwrap();
        let b_filename = b.file_name().unwrap().to_str().unwrap();

        if let (Some(a_num), Some(b_num)) =
            (extract_numeric_prefix(a_filename), extract_numeric_prefix(b_filename))
        {
            a_num.cmp(&b_num)
        } else {
            // Fallback to comparing full filenames if numeric prefix extraction fails
            a_filename.cmp(b_filename)
        }
    });

    paths
}

fn preemptive_remove() {
    let mut child = Command::new("rm")
        .arg("-rf")
        .arg("./post/")
        .spawn()
        .expect("failed to compile");
    let _result = child.wait().unwrap();

    let mut child1 = Command::new("mkdir")
        .arg("./post/")
        .spawn()
        .expect("failed to compile");
    let _result = child1.wait().unwrap();
}

fn build_handler(name: String) {
    let splitted: &Vec<&str> = &name.split(".").collect();

    if splitted.len() != 1 {
        let dir_name = splitted[0];
        let post_name_split: &Vec<&str> = &dir_name.split(")--").collect();
        let mut parent = format!(
            r#"
    <!DOCTYPE html>
    <html class="">
    <head>
    <link href="../../index.css" rel="stylesheet">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.6.0/styles/github-dark.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.6.0/highlight.min.js"></script>
    <!-- Cloudflare Web Analytics --><script defer src='https://static.cloudflareinsights.com/beacon.min.js' data-cf-beacon='{{"token": "e9adb517193447a3a9c4d5064ffa2550"}}'></script><!-- End Cloudflare Web Analytics -->
    <!-- and it's easy to individually load additional languages -->
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.6.0/languages/rust.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.6.0/languages/javascript.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.6.0/languages/python.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.6.0/languages/bash.min.js"></script>
    <script>
    fetch("./index.html").then(resp => resp.text().then(content => {{
        const post = document.createElement("div");
        post.innerHTML =content;
        document.getElementById("post").appendChild(post);
        hljs.highlightAll();
    }}));
    /*
    function resizeIframe(obj) {{
        obj.style.height = obj.contentWindow.document.documentElement.scrollHeight + 'px';
        //obj.contentWindow.document.body.style.color = "white"
        for (el of obj.contentWindow.document.getElementsByTagName("a")) {{
            el.style.color = "\#337AB7"
        }}
    }}
    */
    </script>
    </head>
    <body>
    <div id="post">
    <h4><a href="/">see all posts</a></h1>
    <h1 class="title">{}</h1>
    <!--<iframe src="./index.html" frameborder="0" scrolling="no" width="100%" onload="resizeIframe(this)" />-->
    </div>
        <script>
    function resizeIframe(obj) {{
        obj.style.height = obj.contentWindow.document.documentElement.scrollHeight + 'px';
        obj.style.color = "\#fff"
    }}

    /*fetch("./index.html").then(resp => resp.text().then(content => {{
    const post = document.createElement("div");
    post.innerHTML = /<BODY.*?>([\s\S]*)<\/BODY>/.exec(content)[1];
    document.body.appendChild(post);
    }}));
    */

    </script>
    </body>
    </html>
    "#,
            post_name_split[2].replace("-", " ")
        );

        let mut file = File::create(format!("./post/{}/post.html", &dir_name)).unwrap();
        file.write_all(parent.as_bytes());
    }
}

fn to_html_pd(name: String) {
    let splitted: &Vec<&str> = &name.split(".").collect();

    if splitted.len() != 1 {
        let dir_name = splitted[0];
        let ext_name = splitted[1];
        let mut child1 = Command::new("mkdir")
            .arg(format!("./post/{}", &dir_name))
            .spawn()
            .expect("failed to compile");
        let _result1 = child1.wait().unwrap();

        if ext_name == "tex" {
            let mut child = Command::new("pandoc")
                .arg("-f")
                .arg("latex")
                .arg("-t")
                .arg("html")
                .arg(format!("./drafts/{}", name))
                .arg("-o")
                .arg(format!("./post/{}/index.html", &dir_name))
                .output()
                .expect("failed to compile");
            //    let result = child.wait().unwrap();
            //        let mut out = String::new();
            //        out.push_str(match str::from_utf8(&child.stdout) {
            //            Ok(val) => val,
            //            Err(_) => panic!("got non UTF-8 data from git"),
            //        });
        } else {
            let mut child = Command::new("pandoc")
                .arg("-f")
                .arg("markdown")
                .arg("-t")
                .arg("html")
                .arg(format!("./drafts/{}", name))
                .arg("-o")
                .arg(format!("./post/{}/index.html", &dir_name))
                .output()
                .expect("failed to compile");
            //    let result = child.wait().unwrap();
            //        let mut out = String::new();
            //        out.push_str(match str::from_utf8(&child.stdout) {
            //            Ok(val) => val,
            //            Err(_) => panic!("got non UTF-8 data from git"),
            //        });
        }
        build_handler(name);
    } else {
        let mut child1 = Command::new("mkdir")
            .arg(format!("./post/{}", &name))
            .spawn()
            .expect("failed to compile");
        let _result1 = child1.wait().unwrap();
    }

    
    //    fs::write(format!("./post/{}/index.html", &dir_name), out);
}

fn to_html(name: String) {
    let splitted: &Vec<&str> = &name.split(".").collect();
    let dir_name = splitted[0];
    let mut child = Command::new("latex2html")
        .arg(format!("./drafts/{}", name))
        .spawn()
        .expect("failed to compile");
    let _result = child.wait().unwrap();

    let mut child1 = Command::new("mv")
        .arg(format!("./drafts/{}", dir_name))
        .arg("./post")
        .spawn()
        .expect("failed to compile");
    let _result1 = child1.wait().unwrap();

    build_handler(name);
}

fn gen_index_content(mut posts: Vec<String>) -> String {
    let mut head = String::from(
        r#"<!DOCTYPE html>
<html class="">
<meta charset="UTF-8">
<head> 
<link href="index.css" rel="stylesheet">
<!-- Cloudflare Web Analytics --><script defer src='https://static.cloudflareinsights.com/beacon.min.js' data-cf-beacon='{{"token": "e9adb517193447a3a9c4d5064ffa2550"}}'></script><!-- End Cloudflare Web Analytics -->
</head>
<style>

</style>

"#,
    );

    let mut li_list = String::new();
    let mut posts: Vec<String> = posts.into_iter().rev().collect();
    for post in posts {
        let post_name_split: &Vec<&str> = &post.split(")--").collect();
        
        if post_name_split.len() != 1 {
            let mut dis_name = String::new();

            if &post_name_split[1] == &"pub" {
                dis_name.push_str("&#128215 ");
            } else {
                dis_name.push_str("&#128216 ");
            }

            dis_name.push_str(&post_name_split[2].replace("-", " "));
            li_list.push_str(&format!(
                r#"
    <li>
    <h3><a href="./post/{}/post.html">{}</a></h3>
    </li>
    "#,
                &post, &dis_name
            ))
        } else {
            li_list.push_str(&format!(
                r#"
    <li>
    
    <h3 class="year"><p>{}</p></h3>
    
    </li>
    "#,
                &post.split("--").collect::<Vec<&str>>()[1]
            ))
        }
    }

    let mut ul = String::from(
        r#"
<body>
<div id="content">
<div class="heading">

<h3>Tommaso De Ponti (@heytdep)</h3>

<div class="description">
<p>Hi I'm Tommaso, I am a developer building stuff on the Stellar Network at <a target="_blank" href="https://github.com/xycloo/">Xycloo Labs</a>, and 
currently part-time <a href="https://stellar.org/foundation">Stellar Development Foundation</a> contractor helping to build Soroban-related educational contract-based games 
(<a href="https://fcaooc.com/">fcaooc</a>, <a href="https://quest.stellar.org/">stellar quest</a>, <a href="https://rpciege.com/">rpciege</a>). 
<br/><br/>
My focus lies in decentralized ledger technology, smart contracts, Decentralized Finance, and SaaS. My preferred language is Rust,
but I'm proficient with Javascript, Python, and I'm learning Zig (which is now becoming my second preferred lang).
<br/><br/>
Articles with the &#128215 prefix are intended to be read by a general public, those with the &#128216 prefix are mostly personal notes.</p>

<p>The content on this website is licensed under <a href="https://creativecommons.org/licenses/by/4.0/">Creative Commons Attribution 4.0 International License</a>.</p>
</div>
<br/>
<div id="icons">
    <ul>
        <li><a target="_blank" href="https://github.com/heytdep"><svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 16 16">
        <path
          d="M8 .188c-4.418 0-8 3.582-8 8 0 3.535 2.291 6.527 5.471 7.594.4.074.547-.174.547-.384 0-.191-.007-.876-.012-1.589-2.227.485-2.695-1.061-2.695-1.061-.363-.923-.886-1.169-.886-1.169-.724-.495.055-.486.055-.486.802.057 1.223.824 1.223.824.712 1.218 1.87.867 2.326.663.072-.516.277-.866.503-1.064-1.763-.2-3.617-.881-3.617-3.927 0-.867.31-1.578.824-2.135-.083-.201-.358-1.009.078-2.101 0 0 .666-.213 2.185.815.635-.177 1.313-.266 1.986-.268.672.002 1.35.091 1.985.268 1.52-1.028 2.184-.815 2.184-.815.438 1.092.162 1.9.08 2.101.515.557.823 1.268.823 2.135 0 3.053-1.857 3.724-3.623 3.922.285.246.54.731.54 1.474 0 1.065-.01 1.923-.01 2.191 0 .213.144.463.55.383 3.179-1.067 5.468-4.059 5.468-7.594 0-4.418-3.582-8-8-8z"
        />
      </svg>
      </a></li>
    </ul>
</div>
</div>
<nav class="articles"><ul>"#,
    );
    
    ul.push_str(&li_list);
    ul.push_str("</ul></nav></div></body>");

    head.push_str(&ul);
    head
}

fn write_index() {
    let posts_dirs = get_comp();

    let posts = posts_dirs
        .iter()
        .filter_map(|entry| {
            
                entry.as_path()
                    .file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
            
        })
        .collect::<Vec<String>>();
    
    let content = gen_index_content(posts);
    fs::write("./index.html", content);
    println!("[+] Successfully built posts");
}

fn main() {
    preemptive_remove();
    let drafts = get_drafts();
    
    let posts = drafts
        .iter()
        .filter_map(|entry| {
            
                entry.as_path()
                    .file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
            
        })
        .collect::<Vec<String>>();

    for draft in posts {
        println!("{:?}", draft);
        to_html_pd(draft);
    }

    write_index()
}
