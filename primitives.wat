(module
  ;; WASM module generated from ApexForge NightScript

  ;; Memory section
  (memory 1)
  (export "memory" (memory 0))

  ;; String constants
  (data (i32.const 110) "factorial: ")
  (data (i32.const 150) "is_alphabetic: ")
  (data (i32.const 120) "Hello, AFNS!")
  (data (i32.const 110) "lowercase: ")
  (data (i32.const 150) "String methods:")
  (data (i32.const 80) "length: ")
  (data (i32.const 160) "Integer methods:")
  (data (i32.const 100) "is_prime: ")
  (data (i32.const 110) "uppercase: ")
  (data (i32.const 90) "is_even: ")
  (data (i32.const 180) "Character methods:")

  ;; Helper functions
  (func $string_concat (param $str1 i32) (param $str2 i32) (result i32)
    ;; Simple string concatenation - returns first string for now
    local.get $str1
  )

  (func $println (param $str i32)
    ;; Print string - for now just a no-op
    drop
  )

  ;; Function: apex
  (func $apex
    ;; Variable: num_i8
    i32.const 127
    ;; Variable: num_i32
    i32.const 2147483647
    ;; Variable: num_u32
    i32.const 4294967295
    ;; Variable: num_f32
    f64.const 3.14159
    ;; Variable: num_f64
    f64.const 2.718281828459045
    ;; Variable: flag
    i32.const 1
    ;; Variable: text
    i32.const 120
    ;; Variable: ch
    i32.const 65
    ;; Variable: b
    i32.const 255
    ;; Variable: is_even
    i32.const 0
    ;; Variable: is_prime
    i32.const 0
    ;; Variable: factorial
    i32.const 0
    ;; Variable: length
    i32.const 0
    ;; Variable: uppercase
    i32.const 0
    ;; Variable: is_alphabetic
    i32.const 0
    ;; Variable: lowercase
    i32.const 0
i32.const 160
    call $printlni32.const 90
    i32.const 0
    i32.add
    call $printlni32.const 100
    i32.const 0
    i32.add
    call $printlni32.const 110
    i32.const 0
    i32.add
    call $printlni32.const 150
    call $printlni32.const 80
    i32.const 0
    i32.add
    call $printlni32.const 110
    local.get $uppercase
    i32.add
    call $printlni32.const 180
    call $printlni32.const 150
    i32.const 0
    i32.add
    call $printlni32.const 110
    i32.const 0
    i32.add
    call $println    ;; Implicit return
  )

  ;; Export apex function
  (export "apex" (func $apex))
)
