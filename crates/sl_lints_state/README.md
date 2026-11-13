# sl_lints_state

Lints for shared mutable state, from the fewer-locks rule: every piece of mutable state has one owner (by value, an owner task, a swapped snapshot, the event log, set once), and a lock stays only where it is the right tool, with an item-level reason.

- `static-interior-mutability`: a `static` whose type holds a `Mutex`, `RwLock`, `RefCell`, `Cell` and friends, without a waiver giving the reason.
- `static-mut`: `static mut`.
- `thread-local-state`: `thread_local!` with interior mutability.
- `lock-unwrap`: `.lock().unwrap()` and its kin counted per crate.
- `lock-held-across`: a guard alive at an `.await` or an I/O call.
- `nested-locks`: a second guard taken while one is alive.

Read without type information, so lock calls are recognised by method name; those findings say they are estimates. Depends on `sl_analysis`, `sl_core`, `sl_model` and `syn`.
