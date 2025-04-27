```
                                           _____  _____ 
                                          |____ ||  _  |
 _______ _     _ ______  _______ _    _       / /| |/' |
 |  |  | |____/  |     \ |______  \  /        \ \|  /| |
 |  |  | |    \_ |_____/ |______   \/     .___/ /\ |_/ /
                                          \____(_)\___/ 
```
![image](https://img.shields.io/badge/release-3.2.1-orange)
![image](https://img.shields.io/badge/license-MIT_License-orange)
<a href="https://github.com/4jamesccraven/mkdev/wiki">![image](https://img.shields.io/badge/documentation-🔗-orange)</a>

A CLI Tool for Tinkerers
------------------------
mkdev is a tool for easily generating boilerplate in programs,
scripts, and other projects.

Features
--------
- Automated copying and pasting of directories and their contents
- Simple text substitutions based on user configurations

See the [wiki](https://github.com/4jamesccraven/mkdev/wiki) for more info.

History
------
Mkdev 3.0 is a spiritual successor to 2.0, which was written in python
and functioned far differently. I originally wrote mkdev because I wanted
something for simple scripting like Makefile without writing a new file for
every project. The script was hardcoded, which wasn't ideal for extensibility.
1.2 improved on the idea by making it so users could write configs that would
define the recipes. Unfortunately it was clunky, requiring nested directories
and a poorly conceived config scheme. 2.0 improved on the structure of the config by
flattening it, but this made it difficult for a human to read. In the end I had
to make a custom, buggy built-in tui text editor... very cool, but definitely not
ideal.

So the motivation of this re-write was two-fold: improve the ergononmics of the
program and to use a better-suited and **faster** language.

- [mkdev 1.2](https://github.com/4jamesccraven/mkdev/tree/4d4ac6dd5fe044b7ba3d71d610716b5f3b9685d7)
- [mkdev 2.0](https://github.com/4jamesccraven/mkdev/tree/v2)


Contributing
------------
While I doubt anyone ever will, pull requests are more than welcome.
This project has a special place in my heart as it is the longest
running project I have, and I will probably continue to update it as long as it
interests me.
