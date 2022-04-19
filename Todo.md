- [x] Chartokens
- [x] Nametokens
- [x] Elements (tags) + text content
- [x] attributes
- [x] cdata
- [x] comments
- [x] processing instructions
- [ ] prolog
- [ ] entities
- [ ] post-and-pre root text


- [ ] fix # in comments
- [ ] escape square brackets in rustdoc
- [x] validate references
- [ ] verify end tag equailty while tokenizing with depth()
- [x] only positional errors with range and slice attribute
- [ ] error partialeq
- [ ] error string slices
- [ ] use #NT for doc


Optimize;
- [ ] optimize with byte and single byte checks 
- [ ] compute char_indices() ahead of time
Performance:
- Precompute < locations
- create and use byte methods where applicable
- use match