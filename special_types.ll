; LLVM IR generated from ApexForge NightScript
target triple = "x86_64-unknown-linux-gnu"

@.str.5 = private unnamed_addr constant [6 x i8] c"URL: "
@.str.8 = private unnamed_addr constant [9 x i8] c"Domain: "
@.str.14 = private unnamed_addr constant [15 x i8] c"Current date: "
@.str.5 = private unnamed_addr constant [6 x i8] c"Day: "
@.str.13 = private unnamed_addr constant [14 x i8] c"Email valid: "
@.str.7 = private unnamed_addr constant [8 x i8] c"Email: "
@.str.7 = private unnamed_addr constant [8 x i8] c"Month: "
@.str.12 = private unnamed_addr constant [13 x i8] c"Holo value: "
@.str.12 = private unnamed_addr constant [13 x i8] c"Echo sound: "
@.str.20 = private unnamed_addr constant [21 x i8] c"Portal transferred: "
@.str.4 = private unnamed_addr constant [5 x i8] c"IP: "
@.str.24 = private unnamed_addr constant [25 x i8] c"Timeline current value: "
@.str.17 = private unnamed_addr constant [18 x i8] c"Mirror reflects: "
@.str.12 = private unnamed_addr constant [13 x i8] c"UUID valid: "
@.str.14 = private unnamed_addr constant [15 x i8] c"Trace points: "
@.str.9 = private unnamed_addr constant [10 x i8] c"Version: "
@.str.11 = private unnamed_addr constant [12 x i8] c"Is secure: "
@.str.14 = private unnamed_addr constant [15 x i8] c"Chain length: "
@.str.12 = private unnamed_addr constant [13 x i8] c"Is private: "
@.str.6 = private unnamed_addr constant [7 x i8] c"Year: "
@.str.10 = private unnamed_addr constant [11 x i8] c"Protocol: "
@.str.6 = private unnamed_addr constant [7 x i8] c"UUID: "
@.str.6 = private unnamed_addr constant [7 x i8] c"Host: "
@.str.12 = private unnamed_addr constant [13 x i8] c"Is weekend: "

; Helper functions
declare i8* @string_concat(i8*, i8*)
declare void @println(i8*)

define void @apex() {
  %uuid_test = call void @UUID::new()
i32 0
  %uuid_string = i32 0
  %uuid_validate = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([7 x i8], [7 x i8]* @.str.6, i32 0, i32 0), i8* %uuid_string))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @.str.12, i32 0, i32 0), i8* i32 0))
  %mail = call void @Email::new()
i32 0
  %email_string = i32 0
  %domain = i32 0
  %is_valid = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.7, i32 0, i32 0), i8* %email_string))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @.str.8, i32 0, i32 0), i8* %domain))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([14 x i8], [14 x i8]* @.str.13, i32 0, i32 0), i8* i32 0))
  %url = call void @URL::new()
i32 0
  %protocol = i32 0
  %host = i32 0
  %is_secure = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @.str.5, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([11 x i8], [11 x i8]* @.str.10, i32 0, i32 0), i8* %protocol))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([7 x i8], [7 x i8]* @.str.6, i32 0, i32 0), i8* %host))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([12 x i8], [12 x i8]* @.str.11, i32 0, i32 0), i8* i32 0))
  %ip = call void @IPAddress::new()
i32 0
  %is_private = i32 0
  %version = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str.4, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @.str.12, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([10 x i8], [10 x i8]* @.str.9, i32 0, i32 0), i8* i32 0))
  %date = call void @Date::new()
i32 0
  %year = i32 0
  %month = i32 0
  %day = i32 0
  %is_weekend = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([15 x i8], [15 x i8]* @.str.14, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([7 x i8], [7 x i8]* @.str.6, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([8 x i8], [8 x i8]* @.str.7, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @.str.5, i32 0, i32 0), i8* i32 0))
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @.str.12, i32 0, i32 0), i8* i32 0))
  %timeline = call void @Timeline::new()
i32 0
i32 0
  %current_value = i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([25 x i8], [25 x i8]* @.str.24, i32 0, i32 0), i8* i32 0))
  %holo = call void @Holo::new()
i32 0
i32 0
i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @.str.12, i32 0, i32 0), i8* i32 0))
  %chain = call void @Chain::new()
i32 0
i32 0
i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([15 x i8], [15 x i8]* @.str.14, i32 0, i32 0), i8* i32 0))
  %echo = call void @Echo::new()
i32 0
i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @.str.12, i32 0, i32 0), i8* i32 0))
  %portal = call void @Portal::new()
i32 0
i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([21 x i8], [21 x i8]* @.str.20, i32 0, i32 0), i8* i32 0))
  %original = i32 100
  %mirror = call void @Mirror::new()
i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([18 x i8], [18 x i8]* @.str.17, i32 0, i32 0), i8* i32 0))
  %trace = call void @Trace::new()
i32 0
i32 0
i32 0
call void @println(call i8* @string_concat(i8* getelementptr inbounds ([15 x i8], [15 x i8]* @.str.14, i32 0, i32 0), i8* i32 0))
  ret void
}

