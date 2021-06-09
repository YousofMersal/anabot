import dotenv from 'dotenv'
import {Client} from 'discord.js'
const client = new Client()

dotenv.config()

client.on('ready', () => {
    console.log(`logged in as ${client.user.tag}!`)
})

client.on('message', msg => {
    if (msg.content === 'ping') {
        msg.reply('Pong!')
    }
})

console.log(process.env.TOKEN)

client.login(process.env.TOKEN)
