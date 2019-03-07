use chrono::{DateTime, Utc};
use handlebars::{Context, Handlebars, Helper, JsonRender, Output, RenderContext, RenderError};
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use serde_any;
use std::collections::BTreeMap;
use std::fs;
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize, Deserialize, Clone)]
struct Config {
  site_name: String,
  site_url: String,
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
  subtitle: String,
  date: DateTime<Utc>,
}

fn render_index(site: &Site) -> Result<(), failure::Error> {
  //read the template
  let mut hb = Handlebars::new();
  hb.register_template_file("index", "test-blog/templates/index.hbs")?;
  hb.register_template_file("footer", "test-blog/templates/footer.hbs")?;
  hb.register_template_file("header", "test-blog/templates/header.hbs")?;

  hb.register_helper("date", Box::new(date_helper));

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

fn date_reformatter(date_string: String) -> String {
  let date = date_string.parse::<DateTime<Utc>>().unwrap();
  date.format("%B %e, %Y").to_string()
}

fn date_helper(
  h: &Helper,
  _: &Handlebars,
  _: &Context,
  _: &mut RenderContext,
  out: &mut Output,
) -> Result<(), RenderError> {
  let date_var = h
    .param(0)
    .ok_or_else(|| RenderError::new("Param not found for helper \"date\""))?;
  let date_string = date_var.value().render();
  let date_string_reformatted = date_reformatter(date_string);
  out.write(&date_string_reformatted)?;
  Ok(())
}

fn render_posts(site: &Site) -> Result<(), failure::Error> {
  //read the template
  let mut hb = Handlebars::new();
  hb.register_template_file("post", "test-blog/templates/post.hbs")?;
  hb.register_template_file("footer", "test-blog/templates/footer.hbs")?;
  hb.register_template_file("header", "test-blog/templates/header.hbs")?;

  hb.register_helper("date", Box::new(date_helper));

  for post in site.posts.iter() {
    //render the post with the template
    let post_rendered = hb.render("post", &post)?;

    //save to the build folder
    let write_path = format!("test-blog/build/posts/{}.html", &post.meta.slug);
    fs::write(write_path, post_rendered)?;
  }

  Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
  entry
    .file_name()
    .to_str()
    .map(|s| s.starts_with("."))
    .unwrap_or(false)
}

fn main() -> Result<(), failure::Error> {
  //read config.toml
  let config: Config = serde_any::from_file("test-blog/config.toml")?;

  //collect the .md files and parse into a vec of posts
  let posts: Vec<Post> = WalkDir::new("test-blog/content/posts")
    .into_iter()
    .filter_entry(|e| !is_hidden(e))
    .skip(1)
    .map(|post_path| {
      let path = post_path.unwrap();
      println!("Processing: {}", path.path().display());
      parse_post(path.path().to_str().unwrap(), &config).unwrap()
    })
    .collect();

  //build the site object
  let site = Site {
    config: config,
    posts: posts,
  };

  //render index.html and save to build folder
  render_index(&site)?;

  //render the posts and save them to build folder
  render_posts(&site)?;

  Ok(())
}
