# Rust http server boilerplate, without any specific http frameworks

This is a simple code, written in Rust.

I started to write it for my needs, and at the same time, I wanted to know how helpful it would be to develop with the help of ChatGPT4.
I'll write a blog article on this as soon as possible.

All **latest version of crates** (as of March, the 29th, 2023). Keep it as simple, stupid as possible !

## Specs

* [x] No specific framework for HTTP and routing : **Hyper** and **Tokio** only.
* [x] **Json** support, both for encoding and decoding.
* [x] **Mysql** support for SELECT / INSERT statements.
* [x] Getopts to retrieve commande line parameters.
* [x] Env_logger to log data with customized format.
* [x] Toml to retrieve extra information from config file.
* [x] Serves files from www directory, and specific 404 handler.
* [x] All put in different modules. This might not be as clean as a whistle, but enough to understand how to proceed.

## Todos

* [ ] Upload files
* [ ] GraphQL support
* [ ] Rate limiting and bandwidth control
* [ ] Ssl support
* [ ] Ability to send emails from templates
* [ ] Write a simple documentation explaining how all this works
