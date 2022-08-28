use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::str;

fn get_drafts() -> fs::ReadDir {
    let paths = fs::read_dir("./drafts").unwrap();
    paths
}

fn get_comp() -> fs::ReadDir {
    let paths = fs::read_dir("./comp_posts").unwrap();
    paths
}

fn preemptive_remove() {
    let mut child = Command::new("rm")
        .arg("-rf")
        .arg("./comp_posts/")
        .spawn()
        .expect("failed to compile");
    let _result = child.wait().unwrap();

    let mut child1 = Command::new("mkdir")
        .arg("./comp_posts/")
        .spawn()
        .expect("failed to compile");
    let _result = child1.wait().unwrap();
}

fn build_handler(name: String) {
    let splitted: &Vec<&str> = &name.split(".").collect();
    let dir_name = splitted[0];
    let post_name_split: &Vec<&str> = &dir_name.split(")--").collect();
    let mut parent = format!(
        r#"
<!DOCTYPE html>
<html class="">
<head>
<link href="../../index.css" rel="stylesheet">
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.6.0/styles/default.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.6.0/highlight.min.js"></script>
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
        post_name_split[1].replace("-", " ")
    );

    let mut file = File::create(format!("./comp_posts/{}/post.html", &dir_name)).unwrap();
    file.write_all(parent.as_bytes());
}

fn to_html_pd(name: String) {
    let splitted: &Vec<&str> = &name.split(".").collect();
    let dir_name = splitted[0];

    let mut child1 = Command::new("mkdir")
        .arg(format!("./comp_posts/{}", &dir_name))
        .spawn()
        .expect("failed to compile");
    let _result1 = child1.wait().unwrap();

    let mut child = Command::new("pandoc")
        .arg("-f")
        .arg("latex")
        .arg("-t")
        .arg("html")
        .arg(format!("./drafts/{}", name))
        .output()
        .expect("failed to compile");
    //    let result = child.wait().unwrap();
    let mut out = String::new();
    out.push_str(match str::from_utf8(&child.stdout) {
        Ok(val) => val,
        Err(_) => panic!("got non UTF-8 data from git"),
    });

    fs::write(format!("./comp_posts/{}/index.html", &dir_name), out);

    build_handler(name);
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
        .arg("./comp_posts")
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
</head>
<style>

</style>
"#,
    );

    let mut li_list = String::new();
    posts.sort();
    let rev_posts: Vec<String> = posts.into_iter().rev().collect();

    for post in rev_posts {
        let post_name_split: &Vec<&str> = &post.split(")--").collect();
        li_list.push_str(&format!(
            r#"
<li>
<h3><a href="./comp_posts/{}/post.html">{}</a></h3>
</li>
"#,
            &post,
            &post_name_split[1].replace("-", " ")
        ))
    }

    let mut ul = String::from(
        r#"
<body>
<div id="content">
<div class="heading">
<h1 class="title">tdep's website</h1>
<hr/>
<div class="description">
<p>This is my secondary blog and the place for personal notes, where I write about everyday code challenges, cryptography, smart contracts, and everything that is of a more intermediate/advanced level that doesn't have much traction on <a target="_blank" href="https://tdep.medium.com/">my Medium primary blog</a>.</p>
<p>I built this blog myself starting from a Rust script, which converts LaTeX to blog posts and organizes the webpage's directory. To learn more about my blog read <a href="/comp_posts/0)--How-I-built-this-blog/post.html">this post</a>.</p>
<p>The content on this website is licensed under <a href="https://creativecommons.org/licenses/by/4.0/">Creative Commons Attribution 4.0 International License</a>.</p>
</div>
<div id="icons">
    <ul>
        <li><a target="_blank" href="https://twitter.com/heytdep"><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-twitter" viewBox="0 0 16 16"> <path d="M5.026 15c6.038 0 9.341-5.003 9.341-9.334 0-.14 0-.282-.006-.422A6.685 6.685 0 0 0 16 3.542a6.658 6.658 0 0 1-1.889.518 3.301 3.301 0 0 0 1.447-1.817 6.533 6.533 0 0 1-2.087.793A3.286 3.286 0 0 0 7.875 6.03a9.325 9.325 0 0 1-6.767-3.429 3.289 3.289 0 0 0 1.018 4.382A3.323 3.323 0 0 1 .64 6.575v.045a3.288 3.288 0 0 0 2.632 3.218 3.203 3.203 0 0 1-.865.115 3.23 3.23 0 0 1-.614-.057 3.283 3.283 0 0 0 3.067 2.277A6.588 6.588 0 0 1 .78 13.58a6.32 6.32 0 0 1-.78-.045A9.344 9.344 0 0 0 5.026 15z"/> </svg> </a></li>
        <li><a target="_blank" href="https://tdep.medium.com/"><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-medium" viewBox="0 0 16 16"> <path d="M9.025 8c0 2.485-2.02 4.5-4.513 4.5A4.506 4.506 0 0 1 0 8c0-2.486 2.02-4.5 4.512-4.5A4.506 4.506 0 0 1 9.025 8zm4.95 0c0 2.34-1.01 4.236-2.256 4.236-1.246 0-2.256-1.897-2.256-4.236 0-2.34 1.01-4.236 2.256-4.236 1.246 0 2.256 1.897 2.256 4.236zM16 8c0 2.096-.355 3.795-.794 3.795-.438 0-.793-1.7-.793-3.795 0-2.096.355-3.795.794-3.795.438 0 .793 1.699.793 3.795z"/> </svg></a></li>
        <li><a target="_blank" href="https://tdep.xycloo.com/"><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-house-door-fill" viewBox="0 0 16 16"> <path d="M6.5 14.5v-3.505c0-.245.25-.495.5-.495h2c.25 0 .5.25.5.5v3.5a.5.5 0 0 0 .5.5h4a.5.5 0 0 0 .5-.5v-7a.5.5 0 0 0-.146-.354L13 5.793V2.5a.5.5 0 0 0-.5-.5h-1a.5.5 0 0 0-.5.5v1.293L8.354 1.146a.5.5 0 0 0-.708 0l-6 6A.5.5 0 0 0 1.5 7.5v7a.5.5 0 0 0 .5.5h4a.5.5 0 0 0 .5-.5z"/> </svg></a></li>
    </ul>
</div>
<hr/>
</div>
<div class="articles"><ul>"#,
    );
    ul.push_str(&li_list);
    ul.push_str("</ul></div></div></body>");

    head.push_str(&ul);
    head
}

fn write_index() {
    let posts_dirs = get_comp();

    let posts = posts_dirs
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
            })
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
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
            })
        })
        .collect::<Vec<String>>();

    for draft in posts {
        to_html_pd(draft);
    }

    write_index()
}
