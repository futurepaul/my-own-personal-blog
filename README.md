# my-own-personal-blog

a static site generator

## TODO

- [x] convert markdown to html
- [x] pick a template engine (handlebars)
- [x] read a markdown file and turn it into html
- [x] figure out what type of data handlebars likes (instead of b-tree)
- [x] actually just use structs and derive for everything
- [x] read from a config.toml and store that (for basics like site name)
- [x] read a folder of markdown files and turn them into a folder of html
- [x] add toml frontmatter and turn that into metadata
- [x] render index.html list with set of posts
- [x] render a separate page for each post
- [x] make the rss template and render that too
- [x] robots.txt
- [x] parse dates with a helper function (also in the special rss spec)
- [x] make sure the posts show up in order (also to do with dates)
- [ ] regenerate the entire build dir when run to make sure everything exists
- [ ] copy over the static files
- [ ] init should generate a valid site, including some default templates
