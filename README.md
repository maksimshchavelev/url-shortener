## What is a URL shortener?

This is a link-shortening service. A long link is entered,
and a short code is generated as the output.

## How it works?

A unique 8-digit code is generated for each link and stored
in the database. This code can later be used to retrieve the
original URL.

## What is the project's technology stack?

[Rust](https://rust-lang.org/) is used as the programming language,
along with the main following libraries:

- [axum](https://crates.io/crates/axum) as HTTP framework
- [Tokio](https://crates.io/crates/tokio) as asynchronous runtime
- [Nano ID](https://crates.io/crates/nanoid) as code generator
- [sqlx](https://crates.io/crates/sqlx) for working with database

Other:

- [PostgreSQL](https://en.wikipedia.org/wiki/PostgreSQL) as database
- [Docker](https://en.wikipedia.org/wiki/Docker_(software)) to build an image

## API reference

The [OpenAPI](openapi.yaml) specification is available

## Try shortener

Currently, the URL shortener is available at [s.servix.dev](https://s.servix.dev)

## Using CLI to generate short links

You can use [short](https://github.com/maksimshchavelev/short). 
It's a CLI tool for generating short links from the terminal.

## Building

To build the project, you'll need cargo and the `rustc` compiler,
version `1.96.1`. In the project root directory, run:

```
cargo build --release
```

## Building image using Docker

In the project's root directory, run:

```
docker build -t url-shortener .
```

After that, you can start the `url-shortener` container from the image

## Running

Before running the program, set the following environment variables:

- `SHORTENER_SERVER_PORT` - the port on which the service will be launched
- `SHORTENER_LISTEN_IP` - The IP address on which the service will listen.
  Set this to 0.0.0.0 if you want the service to listen on any IP address.
- `SHORTENER_DATABASE_URL` - URL for connecting to the database. Example:
  `postgres://url_shortener:url_shortener@localhost:5432/url_shortener`
- `RUST_LOG` - Logging Level (optional) - `trace` / `debug` / `info` / `warn` / `error` / `off`

## Running and building via docker compose

Before running the program, set the following environment variables:

- `SHORTENER_DATABASE_USER` (see above)
- `SHORTENER_DATABASE_PASSWORD` (see above)
- `SHORTENER_RUST_LOG` alias for `RUST_LOG` (see above), optional

The service will run on port 4040 (by default) and will listen on the
address 127.0.0.1 (by default)

In the project's root directory, run (be sure to enter your username and password;
do not use the login and password from this example):

```
SHORTENER_DATABASE_USER=admin SHORTENER_DATABASE_PASSWORD=secret docker compose up -d
```

This command can build the image if it hasn't been built yet. If you need to rebuild the 
image, run the command above with the `--build` flag.

> For actual use, it's best to create a separate .env file; otherwise, your passwords 
> might end up in the bash history!

## Development

To set up the development environment, start the database using
`docker-compose-dev.yml`:

```
docker compose -f docker-compose-dev.yml up -d
```

Next, set the following environment variable:

- `DATABASE_URL=postgres://url_shortener:url_shortener@localhost:55432/url_shortener`

After that, apply the migrations (first, install `sqlx` by running `cargo install sqlx-cli`):

```
sqlx migrate run
```

After that, if you don't need a database, you can run `cargo sqlx prepare` and set the
environment variable `SQLX_OFFLINE=true`. Please note that in this case, part of the
process will not be able to run

## Using nginx as proxy

nginx allows you to secure traffic over HTTPS and set a limit on the number 
of requests. 

> Let's say you're setting up a URL shortener on the domain `example.com`

Let's start by configuring proxy settings and limiting the number of requests.
To do this, add the following to the `http` block in the `/etc/nginx/nginx.conf` file:

```
limit_req_zone $binary_remote_addr zone=shortener_limit:10m rate=50r/s;
```

This will limit the number of requests to 50 per second from a single IP address.

Next, create the file `/etc/nginx/sites-available/url-shortener` with the following 
content:

```
upstream shortener_backend {
	server 127.0.0.1:4040;
	keepalive 512;
}

server {
	listen 80;
	server_name example.com; # Change it to your domain

	client_max_body_size 16K;

	location / {
		proxy_pass http://shortener_backend;

		limit_req zone=shortener_limit burst=20 nodelay;
		limit_req_status 429;

		proxy_set_header X-Real-IP $remote_addr;
		proxy_set_header Connection "";

		proxy_http_version 1.1;
	}
}
```

And finally, create a soft link:

```
ln -s /etc/nginx/sites-available/url-shortener /etc/nginx/sites-enabled/url-shortener
```

Great. Now it's time to get a certificate. Let's use Certbot:

```
certbot --nginx -d example.com
```

And finally:

```
systemctl restart nginx
```

You're amazing. nginx is now working as a proxy, wrapping traffic in HTTPS and 
limiting the number of requests.
But don't forget to run the shortener via `docker compose`, or you'll get a 
502 Bad Gateway error.

## License

This project is licensed under the [MIT License](LICENSE)
