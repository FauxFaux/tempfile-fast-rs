### v0.3.5

 * Bump edition and deps.
 * Clarify further that this crate is likely irrelevant.

### v0.3.4

 * Simplify create.
 * Bump edition and deps.

### v0.3.3

 * Add `Sponge` type. 

### v.0.3.2

 * Expose error types.

### v0.3.1

 * Bump deps.

### v0.3.0

 * Actually build on non-linux. :(

### v0.2.0

 * Object, not free function.
 * `persist_noclobber` now returns `PersistError` containing `Self`,
      allowing recovery from failure. 
 * add `persist_by_rename`, utilising the above behaviour.

### v0.1.1

 * Expose internal type name, so users can name it.
