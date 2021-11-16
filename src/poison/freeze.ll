define float @freeze_f32(float %a) unnamed_addr #0 {
  %b = freeze float %a
  ret float %b
}

define double @freeze_f64(double %a) unnamed_addr #0 {
  %b = freeze double %a
  ret double %b
}

define i1 @freeze_i1(i1 %a) unnamed_addr #0 {
  %b = freeze i1 %a
  ret i1 %b
}

define i32 @freeze_i32(i32 %a) unnamed_addr #0 {
  %b = freeze i32 %a
  ret i32 %b
}

define i64 @freeze_i64(i64 %a) unnamed_addr #0 {
  %b = freeze i64 %a
  ret i64 %b
}

attributes #0 = { alwaysinline nofree norecurse willreturn nosync nounwind readnone }
