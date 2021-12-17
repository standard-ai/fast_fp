define float @freeze_f32(float %a) unnamed_addr #0 {
  %b = freeze float %a
  ret float %b
}

define double @freeze_f64(double %a) unnamed_addr #0 {
  %b = freeze double %a
  ret double %b
}

attributes #0 = { alwaysinline nofree norecurse willreturn nosync nounwind readnone }
