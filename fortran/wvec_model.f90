! wvec_model.f90 - Model structure and initialization
module wvec_model
  use, intrinsic :: iso_c_binding
  use wvec_types
  implicit none

  !> Word2Vec model with input and output embeddings
  !> Matrices are (dim, vocab_size) - each column is one word's vector
  type :: W2VModel
    integer(c_int) :: vocab_size = 0
    integer(c_int) :: dim = 0
    real(c_float), allocatable :: w_in(:, :)   ! Input embeddings (dim, vocab_size)
    real(c_float), allocatable :: w_out(:, :)  ! Output embeddings (dim, vocab_size)
  end type W2VModel

contains

  !> Initialize model with random embeddings
  !> Returns 0 on success, negative on error
  function wvec_model_init(model, vocab_size, dim) result(status) bind(C, name="wvec_model_init")
    type(W2VModel), intent(out) :: model
    integer(c_int), intent(in), value :: vocab_size, dim
    integer(c_int) :: status
    integer :: i, j
    real :: rand_val

    ! Validate inputs
    if (vocab_size <= 0 .or. dim <= 0) then
      status = -2  ! ERR_INVALID_SIZE
      return
    end if

    model%vocab_size = vocab_size
    model%dim = dim

    ! Allocate embedding matrices
    allocate (model%w_in(dim, vocab_size), stat=status)
    if (status /= 0) then
      status = -3  ! ERR_OUT_OF_MEMORY
      return
    end if

    allocate (model%w_out(dim, vocab_size), stat=status)
    if (status /= 0) then
      deallocate (model%w_in)
      status = -3  ! ERR_OUT_OF_MEMORY
      return
    end if

    ! Initialize with small random values [-0.5/dim, 0.5/dim]
    do j = 1, vocab_size
      do i = 1, dim
        call random_number(rand_val)
        model%w_in(i, j) = (rand_val - 0.5) / dim
        model%w_out(i, j) = 0.0  ! Output embeddings start at zero
      end do
    end do

    status = 0  ! SUCCESS
  end function wvec_model_init

  !> Free model memory
  subroutine wvec_model_free(model) bind(C, name="wvec_model_free")
    type(W2VModel), intent(inout) :: model

    if (allocated(model%w_in)) deallocate (model%w_in)
    if (allocated(model%w_out)) deallocate (model%w_out)
    model%vocab_size = 0
    model%dim = 0
  end subroutine wvec_model_free

end module wvec_model
