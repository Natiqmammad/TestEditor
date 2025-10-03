; LLVM IR generated from ApexForge NightScript
target triple = "x86_64-unknown-linux-gnu"

@.str.15 = private unnamed_addr constant [16 x i8] c"First element: "
@.str.11 = private unnamed_addr constant [12 x i8] c"Stack top: "
@.str.14 = private unnamed_addr constant [15 x i8] c"Last element: "
@.str.10 = private unnamed_addr constant [11 x i8] c"Set size: "
@.str.13 = private unnamed_addr constant [14 x i8] c"Apple count: "
@.str.12 = private unnamed_addr constant [13 x i8] c"Contains 2: "
@.str.10 = private unnamed_addr constant [11 x i8] c"Dequeued: "
@.str.14 = private unnamed_addr constant [15 x i8] c"Array length: "
@.str.13 = private unnamed_addr constant [14 x i8] c"Queue front: "
@.str.8 = private unnamed_addr constant [9 x i8] c"Popped: "
@.str.11 = private unnamed_addr constant [12 x i8] c"Has grape: "

; Helper functions
declare i8* @string_concat(i8*, i8*)
declare void @println(i8*)

define void @apex() {
  %numbers = call void @Array::new()
i32 0
i32 0
i32 0
  %length = i32 0
  %first = i32 0
  %last = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([15 x i8], [15 x i8]* @.str.14, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([16 x i8], [16 x i8]* @.str.15, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([15 x i8], [15 x i8]* @.str.14, i32 0, i32 0), i8* i32 0))
  %map = call void @Map::new()
i32 0
i32 0
i32 0
  %apple_count = i32 0
  %has_grape = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([14 x i8], [14 x i8]* @.str.13, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.11, i32 0, i32 0), i8* i32 0))
  %set = call void @Set::new()
i32 0
i32 0
i32 0
i32 0
  %contains_two = i32 0
  %set_size = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @.str.12, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([11 x i8], [11 x i8]* @.str.10, i32 0, i32 0), i8* i32 0))
  %queue = call void @Queue::new()
i32 0
i32 0
i32 0
  %front = i32 0
  %dequeued = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([14 x i8], [14 x i8]* @.str.13, i32 0, i32 0), i8* %front))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([11 x i8], [11 x i8]* @.str.10, i32 0, i32 0), i8* %dequeued))
  %stack = call void @Stack::new()
i32 0
i32 0
i32 0
  %top = i32 0
  %popped = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.11, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @.str.8, i32 0, i32 0), i8* i32 0))
  ret void
}

