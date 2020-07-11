# Idle Crawler

![Tests status](https://api.travis-ci.com/s0lst1ce/idle-crawler.svg?token=zSiHNMEiVbeuTV5tCj6L&branch=master&status=started)



Idle crawler came from my (s0lst1ce) desire to create a small terminal-based game in rust. The idea was to hone my rust skills. Eager to share this experience AQUINOS joined some time later.

Although I have a hard time classifying the game I guess it could be called a muiltiplyaer resource management idle game. Indeed players have an empire spanning over large sections of the map. Within their empire they are able to build duiblings which create various goods, that can alter server to make more complex ones. The multiplayer aspects comes from the limits of one's empire. Just like no state can produce everything, the players need to cooperate and trade to be able to get all they need for growth. Whether they wage wars, open trade routes or form alliances is up to them!

An important thing to note is that higher ranked buildings will yield more precious goods, however researching buildings becomes harder and harder, no matter the order in which they are unlocked. Hence it is more viable to specialize and trade with neighbours... or crush them!



## Playing

We will only start building the game for target platforms once it reaches a stable state. By then we will also publish it to https://crates.io/.

### Getting rust

As mentionned before the game is written in rust, a language developed by Mozilla. As most rust developers we use cargo for the packaging. If you don't already have rust installed head over to [its website](https://www.rust-lang.org/learn/get-started) and follow the instructions.

## Building the game

Once you have rust installed and running you should clone the repository by cloning it (or though direct download.). Then enter the `src` directory and the game with cargo.

```bash
git clone git@github.com:s0lst1ce/idle-crawler.git
cd idle-crawler/src
cargo run --bin client
```

## 

## Hosting a game

To host a game you need to open the port you will use for idle crawler. This will not be covered here. You can find many good tutorials on how to do it with a simple search.

Hosting a game is easy and done though the `server` crate. For this simply run the following where `port` is the port you've chosen.

```bash
cargo run --bin server port
```
