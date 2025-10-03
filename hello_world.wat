(module
  ;; WASM module generated from ApexForge NightScript

  ;; Memory section
  (memory 1)
  (export "memory" (memory 0))

  ;; String constants
  (data (i32.const 290) "Hello, ApexForge NightScript!")
  (data (i32.const 70) "Hello, ")
  (data (i32.const 50) "World")
  (data (i32.const 10) "!")

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
    ;; Variable: message
    i32.const 290
local.get $message
    call $println    ;; Implicit return
  )

  ;; Function: greet
  (func $greet (param $name i32) (result i32)
    ;; Variable: greeting
    i32.const 70
    local.get $name
    i32.add
    i32.const 10
    i32.add
    ;; Return
    local.get $greeting
    return
  )

  ;; Function: main
  (func $main
    ;; Variable: name
    i32.const 50
    ;; Variable: greeting
    local.get $name
    call $greet
local.get $greeting
    call $println    ;; Implicit return
  )

  ;; Export main function
  (export "main" (func $main))
  ;; Export apex function
  (export "apex" (func $apex))
)
