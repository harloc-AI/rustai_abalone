# RustAI Abalone

Author: Harald Locke <haraldlocke@gmx.de>

## Summary

Abalone implementation and an agent for playing the game using tensorflow &amp; MCTS based on the concept of Alpha-Zero.
This library is a rust version of [pyai_abalone](https://pypi.org/project/pyai-abalone/)
The reasons for its creation is to run the MCTS at a decent speed without having to handle all the Python issues with threading.

The current version of the crate is rather build for people interested in machine learning and Abalone enthusiasts as it does
not provide any GUI or other meaningful ways to play as a human.

## Future changes

I intend to make it possible to play against the AI.
One of the following options will be released in the future:

* package for Python using PyO3 or similar which can be implemented into the [Abalone3](https://github.com/a-pineau/abalon3/tree/main) GUI
* create a GUI in rust and make a compelte software out of the library
