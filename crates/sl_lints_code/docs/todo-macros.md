## Why

`todo!()` and `unimplemented!()` compile and then panic at run time: they are a promise that the design will be finished later. Under the no-stop-gaps rule a deferred design is a defect; planned work is an entry in FEATURES.md, not a panic waiting in shipped code. Test code is not checked. (Comment markers such as `TODO` and `FIXME` are checked by the prose lints.)

## How to fix

Write the real implementation, or return an error that names what is not supported.
