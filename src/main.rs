use handlebars::Handlebars;
use pulldown_cmark::{html, Parser};
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
  let template = "<!DOCTYPE html>
                    <html>
                      <head>
                        <title>Warp Handlebars template example</title>
                      </head>
                      <body>
                        <h1>Hello {{user}}!</h1>
                        {{{ blogpost }}}
                      </body>
                    </html>";

  let mut hb = Handlebars::new();
  hb.register_template_string("templ.html", template).unwrap();
  // hb.render("templ.html",

  let mut source_file = File::open("test-blog/input/1.md").unwrap();
  let mut markdown_str = String::new();
  source_file.read_to_string(&mut markdown_str)?;

  let parser = Parser::new(&markdown_str);

  let mut html_buf = String::new();
  html::push_html(&mut html_buf, parser);

  let mut data = BTreeMap::new();
  data.insert("user".to_string(), "fartman".to_string());

  for line in html_buf.lines() {
    println!("{}", line);
  }

  data.insert("blogpost".to_string(), html_buf);
  let cool_new_file = hb.render("templ.html", &data).unwrap();

  fs::write("test-blog/output/1.html", cool_new_file)?;

  Ok(())
}
