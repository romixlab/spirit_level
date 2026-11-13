## Why

A function longer than a screen is read in parts and changed without seeing all of it. Long functions are where state machines hide and where edits collide. Test functions are not counted.

## How to fix

Name the steps and make each a function; a long `match` often wants one function per arm.
