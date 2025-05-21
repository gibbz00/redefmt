# Codec

<!-- TODO:

- network (big endian) order
- type hints and how each value is encoded
  - strings and chars are utf8 encoded
  - variable length char even if rust stores it in u32 (UTF32)
  - tuple generation and permutations, wire format length u8
- every which can be determined at compile time is not send over the wire, but instead saved in the db
   - ("{a}", a = 10) sends nothing over the wire but the write ID, literals are stored.
   - reusing variable arguments

    let a = 10; ("{a} {a}", a) does not repeat sending the value `a` over the wire.
        can the macro be smart enough to only send a once here too?
        ```
          let a = 20;
          println!("{a} {x}", a = a, x = a);
        ```


-->
