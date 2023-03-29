# Rust http server boilerplate, without any specific http frameworks

This is a simple code, written in Rust.

## Disclaimer

I started to write it for my needs, and at the same time, I wanted to know how helpful it would be to develop with the help of ChatGPT4.
Please not I did not asked ChatGPT4 to write the whole code, it is too complex to do that. I only asked how to use the crates I wanted to use.
I'll write a blog article on this as soon as possible.

## Why ?

The main idea with this program is to be up to date with the **latest version of all required crates** (as of March, the 29th, 2023).

I also wanted to write something light and as simple stupid as possible ! Thus there is no high level crates such as :

* Rocket
* Actix-web
* Warp
* Tide

All theses crates are *mindblowing* and I **strongly** recommand to use them. But this code is both for learning purpose and the felling of having fine grained control on the functionnalities.

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
* [ ] MongoDB support
* [ ] PostgreSQL support
* [ ] OpenAPI / Swagger support
* [ ] **name yours ...**

## How to use it 

It requires a database. You must first prepare a `users` table with this : 

```mysql
CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255),
    email VARCHAR(255) UNIQUE
);
```

And then copy the file `config.toml.example` to `config.toml`, editing your database parameters :

```toml
[database]
username = "your_username"
password = "your_password"
host = "localhost"
database_name = "your_database_name"
```

Then execute it thanks to cargo :

```bash
cargo r
```

That's it.

I hope this little code will help you. 




