# Anabot
### Raid notification bot 

#### Get started developing
To make the bot work these env variables must be present and set correctly

- DISCORD_TOKEN 
- APPLICATION_ID
- DATABASE_URL

Where `DISCORD_TOKEN` is the token of the discord bot, and where `APPLICATION_ID`,
is the discord application ID. `DATABASE_URL` must be set to connect to a valid PostgreSQL database URI, as of now.
Later plans are made for optional DBMS like MySQL or PostgreSQL and moving the default to SQLite.

If however you don't want to muddle your environment you can make a .env file with the same env variables,
in the format of `DISCORD_TOKEN=<your token>` where`<your token>` is your token, separating them with a newline.
The variables present in this file will override the environment variables for the application only.

#### Host the bot

A [docker hub](https://hub.docker.com/r/yousofmersal/anabot) image is available,
which is updated on every release for easy deployment.
