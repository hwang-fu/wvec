! wvec_types.f90 - Type definitions and constants
module wvec_types
  use, intrinsic :: iso_c_binding
  implicit none

contains

  function wvec_add_smoke_test(a, b) result(c) bind(C, name="wvec_add_smoke_test")
    integer(c_int), intent(in), value :: a, b
    integer(c_int) :: c
    c = a + b
  end function wvec_add_smoke_test

  !> Computes sum of a float array (FFI array passing test)
  function wvec_array_sum(arr, n) result(total) bind(C, name="wvec_array_sum")
    integer(c_int), intent(in), value :: n
    real(c_float), intent(in) :: arr(n)
    real(c_float) :: total
    integer :: i

    total = 0.0
    do i = 1, n
      total = total + arr(i)
    end do
  end function wvec_array_sum

  !> Fills output array with scaled values: out[i] = in[i] * scale
  !> Returns status code (0 = success)
  function wvec_array_scale(arr_in, arr_out, n, scale) result(status) bind(C, name="wvec_array_scale")
    integer(c_int), intent(in), value :: n
    real(c_float), intent(in)         :: arr_in(n)
    real(c_float), intent(out)        :: arr_out(n)
    real(c_float), intent(in), value  :: scale
    integer(c_int)                    :: status
    integer :: i

    ! Validate input
    if (n <= 0) then
      status = -2  ! ERR_INVALID_SIZE
      return
    end if

    do i = 1, n
      arr_out(i) = arr_in(i) * scale
    end do

    status = 0  ! SUCCESS
  end function wvec_array_scale

end module wvec_types
