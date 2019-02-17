### v0.3.0

 * Actually build on non-linux. :(

### v0.2.0

 * Object, not free function.
 * `persist_noclobber` now returns `PersistError` containing `Self`,
      allowing recovery from failure. 
 * add `persist_by_rename`, utilising the above behaviour.

### v0.1.1

 * Expose internal type name, so users can name it.
