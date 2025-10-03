; LLVM IR generated from ApexForge NightScript
target triple = "x86_64-unknown-linux-gnu"

@.str.15 = private unnamed_addr constant [16 x i8] c"String methods:"
@.str.15 = private unnamed_addr constant [16 x i8] c"is_alphabetic: "
@.str.16 = private unnamed_addr constant [17 x i8] c"Integer methods:"
@.str.11 = private unnamed_addr constant [12 x i8] c"uppercase: "
@.str.11 = private unnamed_addr constant [12 x i8] c"factorial: "
@.str.9 = private unnamed_addr constant [10 x i8] c"is_even: "
@.str.8 = private unnamed_addr constant [9 x i8] c"length: "
@.str.10 = private unnamed_addr constant [11 x i8] c"is_prime: "
@.str.12 = private unnamed_addr constant [13 x i8] c"Hello, AFNS!"
@.str.18 = private unnamed_addr constant [19 x i8] c"Character methods:"
@.str.11 = private unnamed_addr constant [12 x i8] c"lowercase: "

; Helper functions
declare i8* @string_concat(i8*, i8*)
declare void @println(i8*)

define void @apex() {
  %num_i8 = i32 127
  %num_i32 = i32 2147483647
  %num_u32 = i32 4294967295
  %num_f32 = double 3.14159
  %num_f64 = double 2.718281828459045
  %flag = i1 1
  %text = getelementptr inbounds ([13 x i8], [13 x i8]* @.str.12, i32 0, i32 0)
  %ch = i8 65
  %b = i32 255
  %is_even = i32 0
  %is_prime = i32 0
  %factorial = i32 0
  %length = i32 0
  %uppercase = i32 0
  %is_alphabetic = i32 0
  %lowercase = i32 0
call void @println(getelementptr inbounds ([17 x i8], [17 x i8]* @.str.16, i32 0, i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([10 x i8], [10 x i8]* @.str.9, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([11 x i8], [11 x i8]* @.str.10, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.11, i32 0, i32 0), i8* i32 0))
call void @println(getelementptr inbounds ([16 x i8], [16 x i8]* @.str.15, i32 0, i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @.str.8, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.11, i32 0, i32 0), i8* %uppercase))
call void @println(getelementptr inbounds ([19 x i8], [19 x i8]* @.str.18, i32 0, i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([16 x i8], [16 x i8]* @.str.15, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.11, i32 0, i32 0), i8* i32 0))
  ret void
}

