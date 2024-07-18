<pre>
 _____ ______   ___  __    ________  _______   ___      ___               _______      ________     
|\   _ \  _   \|\  \|\  \ |\   ___ \|\  ___ \ |\  \    /  /|             /  ___  \    |\   __  \    
\ \  \\\__\ \  \ \  \/  /|\ \  \_|\ \ \   __/|\ \  \  /  / /___________ /__/|_/  /|   \ \  \|\  \   
 \ \  \\|__| \  \ \   ___  \ \  \ \\ \ \  \_|/_\ \  \/  / /\____________\__|//  / /    \ \  \\\  \  
  \ \  \    \ \  \ \  \\ \  \ \  \_\\ \ \  \_|\ \ \    / /\|____________|   /  /_/__  __\ \  \\\  \ 
   \ \__\    \ \__\ \__\\ \__\ \_______\ \_______\ \__/ /                  |\________\\__\ \_______\
    \|__|     \|__|\|__| \|__|\|_______|\|_______|\|__|/                    \|_______\|__|\|_______|
</pre>

![image](https://img.shields.io/badge/release-2.0.2-purple)
![image](https://img.shields.io/badge/license-MIT_License-purple)

Project Description
-------------------
A simple command line tool to setup a development directory from command line.

Features:
---------
- Portable configuration files that define an entire directory structure, deployable with one command.
- Built in config editor to simplify creation and editing of configs
- Fully terminal-based

![edit demo](edit-demo.gif) <br/>
(RIP the quality...)

Usage & Installation:
---------------------
To install run:
```
pip install mkdev-jcc
```
then, run:
```
mkdev init
```
To get an understanding of the config structure, run the following command:
```
mkdev --config-help
```
Motivation & Limitations:
-------------------------
When learning Make in my computer science classes, I really liked the utilities that I saw some people
build into them (like making directories, removing binaries to clean, etc.). I wanted that functionality
for other languages (not to imply this isn't the case with make), and I wanted it to be cross-platform
(no more `make cleanwin`!). This project started as a glorified shell script written in python, but grew
into something more as some of my friends became interested in using the software. It's not really meant
to be software for the general public, though it is open for said use. This is mostly something that I
wrote for fun.

Acknowledgements:
-----------------
I want to give my thanks to:
- Garrett, for being interested
- [Steven](https://github.com/Steven-S1020), for encouraging me to continue, and being excited for me.
- [Dad](https://github.com/eagle79), for listening to me ramble non-sensically about my nix derivation, and why it wouldn't build.
- [Trauma](https://github.com/t-v), for teaching me a lot about writing idiomatic python, and answering many a stupid question.
- [This site](https://patorjk.com/software/taag) for the ascii art.
