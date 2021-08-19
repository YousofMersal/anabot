# Anabot
### Chronicles raid notification bot 

#### Get started developing

To connect the bot with a discord application create a `.env` file at the root of the project.
In this file add the line `TOKEN=<your token>` and replace `<your token>` with your discord bot token.


##### Unusual Dependency
This repo depends on [Scheduler](https://git.yousoftware.live/Yousof/Scheduler) a modified version
of and async cron scheduler library to work with system local time instead of UTC. It can be found [here](https://git.yousoftware.live/Yousof/Scheduler)
hosted on my git server. Download it and place it somewhere on your system, remember to correct the relative path in the Cargo.toml if you do.
