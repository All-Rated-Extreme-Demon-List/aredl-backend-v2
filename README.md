# aredl-backend-v2

Welcome to the repository for the [All Rated Extreme Demons List](https://aredl.net/) backend! This codebase uses Rust, the [Actix framework](https://actix.rs/docs/), and PostgreSQL. Feel free to report bugs, suggest features and/or create pull requests!

# How to run the API locally

## Step 1: Install everything

For this guide you will need to install:

* [a Rust toolchain](https://rustup.rs/)
* [Docker](https://docs.docker.com/engine/install/)
* [Postgres](https://www.postgresql.org/download/)
* [Git](https://git-scm.com/downloads) (if you don’t have it already)


After that, clone this repository onto your computer:

```bash
cd ./path/to/repo/
git clone https://github.com/All-Rated-Extreme-Demon-List/aredl-backend-v2.git
cd ./aredl-backend-v2/
```


## Step 2: Environment Variables

Next, create a [Discord Developer application](https://discord.com/developers/applications), and go into the OAuth page on the left. Under “Redirects” set the URI to `http://127.0.0.1:5000/api/auth/discord/callback`.


Next, duplicate the `.env.example` file in the project’s root, rename it to `.env`, and fill out all the variables inside. Some values are already filled in for you, and you usually don’t need to change them. If you change the API host or port, remember to also change the Redirect URI for your Discord Developer application to match it.


## Step 3: Starting the server

The production environment uses Docker to host the API and database, so it’s suggested that you do too. To start the Docker containers, run…

```bash
docker compose up --build
```

…in the project root.

If everything is setup correctly, the server should startup and be available at `http://127.0.0.1:5000`!
