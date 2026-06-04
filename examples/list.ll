; ModuleID = 'main'
source_filename = "main"

@.fmt_int = global [4 x i8] c"%ld\00"
@.fmt_float = global [4 x i8] c"%g \00"
@.fmt_str = global [3 x i8] c"%s\00"
@.fmt_nl = global [2 x i8] c"\0A\00"
@.str_true = global [5 x i8] c"true\00"
@.str_false = global [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)

declare ptr @malloc(i64)

define void @atomic_print_int(i64 %0) {
entry:
  %1 = call i32 (ptr, ...) @printf(ptr @.fmt_int, i64 %0)
  ret void
}

define void @atomic_print_float(double %0) {
entry:
  %1 = call i32 (ptr, ...) @printf(ptr @.fmt_float, double %0)
  ret void
}

define void @atomic_print_bool(i1 %0) {
entry:
  br i1 %0, label %true_branch, label %false_branch

true_branch:                                      ; preds = %entry
  %1 = call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @.str_true)
  ret void

false_branch:                                     ; preds = %entry
  %2 = call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr @.str_false)
  ret void
}

define void @atomic_print_string({ i64, ptr } %0) {
entry:
  %data = extractvalue { i64, ptr } %0, 1
  %1 = call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr %data)
  ret void
}

define void @atomic_println() {
entry:
  %0 = call i32 (ptr, ...) @printf(ptr @.fmt_nl)
  ret void
}

define { i64, ptr } @atomic_string_create(ptr %0, i64 %1) {
entry:
  %alloc_size = add i64 %1, 1
  %buf = call ptr @malloc(i64 %alloc_size)
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %buf, ptr align 1 %0, i64 %1, i1 false)
  %null_pos = getelementptr i8, ptr %buf, i64 %1
  store i8 0, ptr %null_pos, align 1
  %r1 = insertvalue { i64, ptr } undef, i64 %1, 0
  %r2 = insertvalue { i64, ptr } %r1, ptr %buf, 1
  ret { i64, ptr } %r2
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias nocapture writeonly, ptr noalias nocapture readonly, i64, i1 immarg) #0

define { i64, ptr } @atomic_string_concat({ i64, ptr } %0, { i64, ptr } %1) {
entry:
  %len1 = extractvalue { i64, ptr } %0, 0
  %data1 = extractvalue { i64, ptr } %0, 1
  %len2 = extractvalue { i64, ptr } %1, 0
  %data2 = extractvalue { i64, ptr } %1, 1
  %total = add i64 %len1, %len2
  %buf = call ptr @malloc(i64 %total)
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %buf, ptr align 1 %data1, i64 %len1, i1 false)
  %offset = getelementptr i8, ptr %buf, i64 %len1
  call void @llvm.memcpy.p0.p0.i64(ptr align 1 %offset, ptr align 1 %data2, i64 %len2, i1 false)
  %r1 = insertvalue { i64, ptr } undef, i64 %total, 0
  %r2 = insertvalue { i64, ptr } %r1, ptr %buf, 1
  ret { i64, ptr } %r2
}

define { ptr, i64, i64 } @atomic_list_create(i64 %0) {
entry:
  %alloc_size = mul i64 %0, 8
  %data = call ptr @malloc(i64 %alloc_size)
  %r1 = insertvalue { ptr, i64, i64 } undef, ptr %data, 0
  %r2 = insertvalue { ptr, i64, i64 } %r1, i64 0, 1
  %r3 = insertvalue { ptr, i64, i64 } %r2, i64 %0, 2
  ret { ptr, i64, i64 } %r3
}

define { ptr, i64, i64 } @atomic_list_push({ ptr, i64, i64 } %0, i64 %1) {
entry:
  %data = extractvalue { ptr, i64, i64 } %0, 0
  %len = extractvalue { ptr, i64, i64 } %0, 1
  %cap = extractvalue { ptr, i64, i64 } %0, 2
  %elem_ptr = getelementptr i64, ptr %data, i64 %len
  store i64 %1, ptr %elem_ptr, align 4
  %new_len = add i64 %len, 1
  %r1 = insertvalue { ptr, i64, i64 } undef, ptr %data, 0
  %r2 = insertvalue { ptr, i64, i64 } %r1, i64 %new_len, 1
  %r3 = insertvalue { ptr, i64, i64 } %r2, i64 %cap, 2
  ret { ptr, i64, i64 } %r3
}

define i64 @atomic_list_get({ ptr, i64, i64 } %0, i64 %1) {
entry:
  %data = extractvalue { ptr, i64, i64 } %0, 0
  %elem_ptr = getelementptr i64, ptr %data, i64 %1
  %val = load i64, ptr %elem_ptr, align 4
  ret i64 %val
}

define i64 @atomic_list_len({ ptr, i64, i64 } %0) {
entry:
  %len = extractvalue { ptr, i64, i64 } %0, 1
  ret i64 %len
}

define void @main() {
entry:
  %0 = call { ptr, i64, i64 } @atomic_list_create(i64 5)
  %list_tmp = alloca { ptr, i64, i64 }, align 8
  store { ptr, i64, i64 } %0, ptr %list_tmp, align 8
  %list_load = load { ptr, i64, i64 }, ptr %list_tmp, align 8
  %1 = call { ptr, i64, i64 } @atomic_list_push({ ptr, i64, i64 } %list_load, i64 1)
  store { ptr, i64, i64 } %1, ptr %list_tmp, align 8
  %list_load1 = load { ptr, i64, i64 }, ptr %list_tmp, align 8
  %2 = call { ptr, i64, i64 } @atomic_list_push({ ptr, i64, i64 } %list_load1, i64 2)
  store { ptr, i64, i64 } %2, ptr %list_tmp, align 8
  %list_load2 = load { ptr, i64, i64 }, ptr %list_tmp, align 8
  %3 = call { ptr, i64, i64 } @atomic_list_push({ ptr, i64, i64 } %list_load2, i64 3)
  store { ptr, i64, i64 } %3, ptr %list_tmp, align 8
  %list_load3 = load { ptr, i64, i64 }, ptr %list_tmp, align 8
  %4 = call { ptr, i64, i64 } @atomic_list_push({ ptr, i64, i64 } %list_load3, i64 4)
  store { ptr, i64, i64 } %4, ptr %list_tmp, align 8
  %list_load4 = load { ptr, i64, i64 }, ptr %list_tmp, align 8
  %5 = call { ptr, i64, i64 } @atomic_list_push({ ptr, i64, i64 } %list_load4, i64 5)
  store { ptr, i64, i64 } %5, ptr %list_tmp, align 8
  %nums = alloca { ptr, i64, i64 }, align 8
  store ptr %list_tmp, ptr %nums, align 8
  call void @atomic_print_int(i64 1)
  %nums5 = load { ptr, i64, i64 }, ptr %nums, align 8
  %list_tmp6 = alloca { ptr, i64, i64 }, align 8
  store { ptr, i64, i64 } %nums5, ptr %list_tmp6, align 8
  %list_load7 = load { ptr, i64, i64 }, ptr %list_tmp6, align 8
  %6 = call i64 @atomic_list_get({ ptr, i64, i64 } %list_load7, i64 0)
  %first = alloca i64, align 8
  store i64 %6, ptr %first, align 4
  %first8 = load i64, ptr %first, align 4
  call void @atomic_print_int(i64 %first8)
  %nums9 = load { ptr, i64, i64 }, ptr %nums, align 8
  %list_tmp10 = alloca { ptr, i64, i64 }, align 8
  store { ptr, i64, i64 } %nums9, ptr %list_tmp10, align 8
  %list_load11 = load { ptr, i64, i64 }, ptr %list_tmp10, align 8
  %7 = call i64 @atomic_list_get({ ptr, i64, i64 } %list_load11, i64 2)
  %third = alloca i64, align 8
  store i64 %7, ptr %third, align 4
  %third12 = load i64, ptr %third, align 4
  call void @atomic_print_int(i64 %third12)
  call void @atomic_print_int(i64 9)
  ret void
}

attributes #0 = { nocallback nofree nounwind willreturn memory(argmem: readwrite) }
