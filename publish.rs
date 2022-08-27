use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

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
    let mut parent = format!(
        r#"
<!DOCTYPE html>
<html class="dark">
<head>
<link href="../../index.css" rel="stylesheet">
<script>
function resizeIframe(obj) {{
    obj.style.height = obj.contentWindow.document.documentElement.scrollHeight + 'px';
    obj.contentWindow.document.body.style.color = "white"
    for (el of obj.contentWindow.document.getElementsByTagName("a")) {{
        el.style.color = "\#77e1cd"
    }}
  }}

</script>
</head>
<body>
<h1><a href="/">tdep's blog</a></h1>
<div id="post">
<iframe src="./index.html" frameborder="0" scrolling="no" width="100%" onload="resizeIframe(this)" />
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
"#
    );

    let mut file = File::create(format!("./comp_posts/{}/post.html", &dir_name)).unwrap();
    file.write_all(parent.as_bytes());
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

fn gen_index_content(posts: Vec<String>) -> String {
    let mut head = String::from(
        r#"<!DOCTYPE html>
<html class="dark">
<meta charset="UTF-8">
<head> 
<link href="index.css" rel="stylesheet">
</head>
<style>

</style>
"#,
    );

    let mut li_list = String::new();

    for post in posts {
        li_list.push_str(&format!(
            r#"
<li>
<h3><a href="./comp_posts/{}/post.html">{}</a></h3>
</li>
"#,
            &post,
            &post.replace("-", " ")
        ))
    }

    let mut ul = String::from(
        r#"
<body>
<div class="heading">
<h1><a href="/">tdep's blog</a></h1>
<div class="description">
<p>This is my secondary blog, where I write about everyday code challenges, cryptography, smart contracts, and everything that is of a more intermediate/advanced level that don't have much traction in <a target="_blank" href="https://tdep.medium.com/">my primary blog</a></p>
</div>
</div>
<div class="articles"><ul>"#,
    );
    ul.push_str(&li_list);
    ul.push_str("</ul></div></body>");

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
    println!("{:?}", posts);
    let content = gen_index_content(posts);
    fs::write("./index.html", content);
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
        to_html(draft);
    }

    write_index()
}
