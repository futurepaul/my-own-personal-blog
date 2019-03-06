use handlebars::Handlebars;
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use serde_any;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
  site_name: String,
}

#[derive(Serialize, Deserialize)]
struct Site {
  config: Config,
  posts: Vec<Post>,
}

#[derive(Serialize, Deserialize)]
struct Post {
  meta: PostMeta,
  config: Config,
  content: String,
}

#[derive(Serialize, Deserialize)]
struct PostMeta {
  title: String,
  slug: String,
  date: String,
}

fn render_index(site: &Site) -> Result<(), failure::Error> {
  //read the template
  let mut hb = Handlebars::new();
  hb.register_template_file("index", "test-blog/templates/index.html.hbs")?;

  //render the config data with the template
  let index_rendered = hb.render("index", site)?;

  //save to the build folder
  fs::write("test-blog/build/index.html", index_rendered)?;

  Ok(())
}

fn parse_post(source: &str, config: &Config) -> Result<Post, failure::Error> {
  //read the post from the provided path
  let post_source_md = fs::read_to_string(source)?;

  //split on +++ and skip the first one which is empty
  let post_vec = post_source_md.split("+++").skip(1).collect::<Vec<&str>>();

  //parse the frontmatter toml
  let post_meta: PostMeta = serde_any::from_str(&post_vec[0], serde_any::Format::Toml)?;

  //parse the markdown
  let parser = Parser::new(&post_vec[1]);
  let mut html_buf = String::new();
  html::push_html(&mut html_buf, parser);

  Ok(Post {
    meta: post_meta,
    config: config.clone(),
    content: html_buf,
  })
}

fn posts_to_files(site: &Site) -> Result<(), failure::Error> {
  //read the template
  let mut hb = Handlebars::new();
  hb.register_template_file("post", "test-blog/templates/post.html.hbs")?;

  for post in site.posts.iter() {
    //render the post with the template
    let post_rendered = hb.render("post", &post)?;

    //save to the build folder
    let write_path = format!("test-blog/build/posts/{}.html", &post.meta.slug);
    fs::write(write_path, post_rendered)?;
  }

  Ok(())
}

fn main() -> Result<(), failure::Error> {
  //read config.toml
  let config: Config = serde_any::from_file("test-blog/config.toml")?;

  let posts: Vec<Post> = WalkDir::new("test-blog/content/posts")
    .into_iter()
    .skip(1)
    .map(|post_path| parse_post(post_path.unwrap().path().to_str().unwrap(), &config).unwrap())
    .collect();

  // for post in WalkDir::new("test-blog/content/posts").into_iter().skip(1) {
  //   println!("{}", post?.path().display());
  // }

  let site = Site {
    config: config,
    posts: posts,
  };

  render_index(&site)?;
  posts_to_files(&site)?;

  Ok(())
}
