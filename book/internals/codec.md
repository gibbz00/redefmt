# Codec

<!-- TODO:

- network (big endian) order
- type hints and how each value is encoded
  - strings and chars are utf8 encoded
  - variable length char even if rust stores it in u32 (UTF32)
  - tuple generation and permutations, wire format length u8
- every which can be determined at compile time is not send over the wire, but instead saved in the db
  - provided arg literals aren't encoded, but saved in the database
  - TODO: mentions some takeaways from `ArgumentsResolver::resolve`
-->
