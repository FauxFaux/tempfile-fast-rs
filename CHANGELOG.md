### v0.2.0

 * `persist_noclobber` now returns `PersistError` containing `Self`,
      allowing recovery from failure. 
 * add `persist_by_rename`, utilising the above behaviour.

### v0.1.1

 * Expose internal type name, so users can name it.