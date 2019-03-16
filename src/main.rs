use chrono::{DateTime, Utc};
use clap::{App, Arg};
use handlebars::{Context, Handlebars, Helper, JsonRender, Output, RenderContext, RenderError};
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use serde_any;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Serialize, Deserialize, Clone)]
struct Config {
  site_name: String,
  site_url: String,
  author: String,
  email: String,
  disallow: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Site {
  config: Config,
  posts: Vec<Post>,
  root: String,
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
  hb.register_template_file("index", join_root(&site.root, "templates/index.hbs"))?;
  hb.register_template_file("footer", join_root(&site.root, "templates/footer.hbs"))?;
  hb.register_template_file("header", join_root(&site.root, "templates/header.hbs"))?;

  hb.register_helper("date", Box::new(date_helper));

  //render the config data with the template
  let index_rendered = hb.render("index", site)?;

  //save to the build folder
  fs::write(join_root(&site.root, "build/index.html"), index_rendered)?;

  Ok(())
}

fn render_robots(site: &Site) -> Result<(), failure::Error> {
  //read the template
  let mut hb = Handlebars::new();
  hb.register_template_file("robots", join_root(&site.root, "templates/robots.txt.hbs"))?;

  //render the config data with the template
  let robots_rendered = hb.render("robots", site)?;

  //save to the build folder
  fs::write(join_root(&site.root, "build/robots.txt"), robots_rendered)?;

  Ok(())
}

fn render_rss(site: &Site) -> Result<(), failure::Error> {
  //read the template
  let mut hb = Handlebars::new();
  hb.register_template_file("rss", join_root(&site.root, "templates/rss.xml.hbs"))?;

  hb.register_helper("date_rss", Box::new(date_rss_helper));

  //render the config data with the template
  let rss_rendered = hb.render("rss", site)?;

  //save to the build folder
  fs::write(join_root(&site.root, "build/rss.xml"), rss_rendered)?;

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

fn date_rss_reformatter(date_string: String) -> String {
  let date = date_string.parse::<DateTime<Utc>>().unwrap();
  date.to_rfc2822()
}

fn date_rss_helper(
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
  let date_string_reformatted = date_rss_reformatter(date_string);
  out.write(&date_string_reformatted)?;
  Ok(())
}

fn render_posts(site: &Site) -> Result<(), failure::Error> {
  //read the template
  let mut hb = Handlebars::new();
  hb.register_template_file("post", join_root(&site.root, "templates/post.hbs"))?;
  hb.register_template_file("footer", join_root(&site.root, "templates/footer.hbs"))?;
  hb.register_template_file("header", join_root(&site.root, "templates/header.hbs"))?;

  hb.register_helper("date", Box::new(date_helper));

  for post in site.posts.iter() {
    //render the post with the template
    let post_rendered = hb.render("post", &post)?;

    //save to the build folder
    let write_path = join_root(&site.root, &format!("build/posts/{}.html", &post.meta.slug));
    fs::write(write_path, post_rendered)?;
  }

  Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
  entry
    .file_name()
    .to_str()
    .map(|s| s.starts_with('.'))
    .unwrap_or(false)
}

fn generate_dirs(path: &str) -> Result<(), failure::Error> {
  let mut builder = fs::DirBuilder::new();
  builder.recursive(true);
  builder.create(format!("{}/templates", path))?;
  builder.create(format!("{}/content/static", path))?;

  Ok(())
}

fn join_root(root: &str, rest: &str) -> PathBuf {
  Path::new(root).join(rest)
}

fn main() -> Result<(), failure::Error> {
  let args = App::new("blog")
    .arg(
      Arg::with_name("init")
        .long("init")
        .value_name("PATH")
        .help("Specifies the name of the folder containing site info")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("root")
        .long("root")
        .value_name("PATH")
        .help("Where you want to run this command")
        .takes_value(true),
    )
    .get_matches();

  if let Some(path) = args.value_of("init") {
    generate_dirs(path)?;
  }

  let mut root_path = "";

  if let Some(path) = args.value_of("root") {
    println!("root is: {}", path);
    root_path = path;
  }

  //read config.toml
  let config: Config = serde_any::from_file(join_root(root_path, "config.toml"))?;

  //collect the .md files and parse into a vec of posts
  let mut posts: Vec<Post> = WalkDir::new(join_root(root_path, "content/posts"))
    .into_iter()
    .filter_entry(|e| !is_hidden(e))
    .skip(1)
    .map(|post_path| {
      let path = post_path.unwrap();
      println!("Processing: {}", path.path().display());
      parse_post(path.path().to_str().unwrap(), &config).unwrap()
    })
    .collect();

  //sort posts by date (reverse chronologically)
  posts.sort_unstable_by(|a, b| b.meta.date.cmp(&a.meta.date));

  //build the site object
  let site = Site {
    config,
    posts,
    root: root_path.to_string(),
  };

  //render index.html and save to build folder
  render_index(&site)?;

  //render the posts and save them to build folder
  render_posts(&site)?;

  //render rss.xml and...
  render_rss(&site)?;

  //render robotx.txt and...
  render_robots(&site)?;

  Ok(())
}
