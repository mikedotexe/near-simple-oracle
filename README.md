This is a work-in-progress oracle system.
The built-out parts are the oracle-contract and oracle-nodes/trusted-1

There are no external calls yet, just set up the system to register oracle nodes and left off with some commit-reveal.

The purpose of oracle-nodes is to be able to run these to simulate an oracle node by running `node oracle-node.js` and eventually to simulate a bad actor with `node oracle-node.js evil`

Some API keys for a couple sites are in NEAR's shared LastPass, but have yet to reach out to the external world.