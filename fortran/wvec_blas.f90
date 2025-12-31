! wvec_blas.f90 - BLAS interfaces for linear algebra operations
module wvec_blas
  use, intrinsic :: iso_c_binding
  implicit none

  ! BLAS Level 1 interfaces (vector operations)
  interface
    !> Dot product: result = x · y
    real(c_float) function sdot(n, x, incx, y, incy)
      import :: c_float, c_int
      integer(c_int), intent(in), value :: n, incx, incy
      real(c_float), intent(in) :: x(*), y(*)
    end function sdot
  end interface

end module wvec_blas
