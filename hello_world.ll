; LLVM IR generated from ApexForge NightScript
target triple = "x86_64-unknown-linux-gnu"

@.str.29 = private unnamed_addr constant [30 x i8] c"Hello, ApexForge NightScript!"
@.str.1 = private unnamed_addr constant [2 x i8] c"!"
@.str.7 = private unnamed_addr constant [8 x i8] c"Hello, "
@.str.5 = private unnamed_addr constant [6 x i8] c"World"

; Helper functions
declare i8* @string_concat(i8*, i8*)
declare void @println(i8*)

define void @apex() {
  %message = getelementptr inbounds ([30 x i8], [30 x i8]* @.str.29, i32 0, i32 0)
call void @println(%message)
  ret void
}

define i8* @greet(i8* %name) {
  %greeting = call i8* @string_concat(i8* call i8* @string_concat(i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.7, i32 0, i32 0), i8* %name), i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.str.1, i32 0, i32 0))
  ret %greeting
}

define void @main() {
  %name = getelementptr inbounds ([6 x i8], [6 x i8]* @.str.5, i32 0, i32 0)
  %greeting = call i8* @greet(%name)
call void @println(%greeting)
  ret void
}

