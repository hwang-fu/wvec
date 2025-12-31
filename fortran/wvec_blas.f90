! wvec_blas.f90 - BLAS interfaces for linear algebra operations
module wvec_blas
  use, intrinsic :: iso_c_binding
  implicit none

  ! BLAS Level 1 interfaces (vector operations)
  ! Note: Standard BLAS passes scalars by reference
  interface
    !> Dot product: result = x · y
    real function sdot(n, x, incx, y, incy)
      integer, intent(in) :: n, incx, incy
      real, intent(in) :: x(*), y(*)
    end function sdot

    !> Vector update: y = alpha * x + y
    subroutine saxpy(n, alpha, x, incx, y, incy)
      integer, intent(in) :: n, incx, incy
      real, intent(in) :: alpha
      real, intent(in) :: x(*)
      real, intent(inout) :: y(*)
    end subroutine saxpy

    !> L2 norm: result = ||x||_2
    real function snrm2(n, x, incx)
      integer, intent(in) :: n, incx
      real, intent(in) :: x(*)
    end function snrm2

    !> Scale vector: x = alpha * x
    subroutine sscal(n, alpha, x, incx)
      integer, intent(in) :: n, incx
      real, intent(in) :: alpha
      real, intent(inout) :: x(*)
    end subroutine sscal
  end interface

end module wvec_blas
