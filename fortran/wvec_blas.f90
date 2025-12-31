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

    !> Vector update: y = alpha * x + y
    subroutine saxpy(n, alpha, x, incx, y, incy)
      import :: c_float, c_int
      integer(c_int), intent(in), value :: n, incx, incy
      real(c_float), intent(in), value :: alpha
      real(c_float), intent(in) :: x(*)
      real(c_float), intent(inout) :: y(*)
    end subroutine saxpy

    !> L2 norm: result = ||x||_2
    real(c_float) function snrm2(n, x, incx)
      import :: c_float, c_int
      integer(c_int), intent(in), value :: n, incx
      real(c_float), intent(in) :: x(*)
    end function snrm2

    !> Scale vector: x = alpha * x
    subroutine sscal(n, alpha, x, incx)
      import :: c_float, c_int
      integer(c_int), intent(in), value :: n, incx
      real(c_float), intent(in), value :: alpha
      real(c_float), intent(inout) :: x(*)
    end subroutine sscal
  end interface

end module wvec_blas
