! wvec_types.f90 - Type definitions and constants
module wvec_types
  use, intrinsic :: iso_c_binding
  implicit none

  ! TODO: define model params, array types, etc.

contains

  function wvec_add_smoke_test(a, b) result(c) bind(C, name="wvec_add_smoke_test")
    integer(c_int), intent(in), value :: a, b
    integer(c_int) :: c
    c = a + b
  end function wvec_add_smoke_test

end module wvec_types
