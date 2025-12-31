! wvec_model.f90 - Model structure and initialization
module wvec_model
  use, intrinsic :: iso_c_binding
  use wvec_types
  implicit none

  ! Module-level model (singleton)
  integer(c_int), save :: g_vocab_size = 0
  integer(c_int), save :: g_dim = 0
  real(c_float), allocatable, save :: g_w_in(:, :)   ! Input embeddings (dim, vocab_size)
  real(c_float), allocatable, save :: g_w_out(:, :)  ! Output embeddings (dim, vocab_size)
  logical, save :: g_initialized = .false.

  !> Shutdown flag for graceful termination
  logical, save :: g_shutdown_requested = .false.

contains

  !> Initialize model with random embeddings
  !> Returns 0 on success, negative on error
  function wvec_model_init(vocab_size, dim) result(status) bind(C, name="wvec_model_init")
    integer(c_int), intent(in), value :: vocab_size, dim
    integer(c_int) :: status
    integer :: i, j, alloc_stat
    real :: rand_val

    ! Free existing model if any
    call wvec_model_free()

    ! Validate inputs
    if (vocab_size <= 0 .or. dim <= 0) then
      status = -2  ! ERR_INVALID_SIZE
      return
    end if

    g_vocab_size = vocab_size
    g_dim = dim

    ! Allocate embedding matrices
    allocate (g_w_in(dim, vocab_size), stat=alloc_stat)
    if (alloc_stat /= 0) then
      status = -3  ! ERR_OUT_OF_MEMORY
      return
    end if

    allocate (g_w_out(dim, vocab_size), stat=alloc_stat)
    if (alloc_stat /= 0) then
      deallocate (g_w_in)
      status = -3  ! ERR_OUT_OF_MEMORY
      return
    end if

    ! Initialize with small random values [-0.5/dim, 0.5/dim]
    do j = 1, vocab_size
      do i = 1, dim
        call random_number(rand_val)
        g_w_in(i, j) = (rand_val - 0.5) / dim
        call random_number(rand_val)
        g_w_out(i, j) = (rand_val - 0.5) / dim  ! Also randomize output embeddings
      end do
    end do

    g_initialized = .true.
    status = 0  ! SUCCESS
  end function wvec_model_init

  !> Free model memory
  subroutine wvec_model_free() bind(C, name="wvec_model_free")
    if (allocated(g_w_in)) deallocate (g_w_in)
    if (allocated(g_w_out)) deallocate (g_w_out)
    g_vocab_size = 0
    g_dim = 0
    g_initialized = .false.
  end subroutine wvec_model_free

  !> Get model dimensions
  subroutine wvec_model_get_dims(vocab_size, dim) bind(C, name="wvec_model_get_dims")
    integer(c_int), intent(out) :: vocab_size, dim
    vocab_size = g_vocab_size
    dim = g_dim
  end subroutine wvec_model_get_dims

  !> Check if model is initialized
  function wvec_model_is_init() result(is_init) bind(C, name="wvec_model_is_init")
    integer(c_int) :: is_init
    if (g_initialized) then
      is_init = 1
    else
      is_init = 0
    end if
  end function wvec_model_is_init

  !> Copy embedding for word_id to output buffer (0-indexed)
  function wvec_get_embedding(word_id, out_vec, out_len) result(status) bind(C, name="wvec_get_embedding")
    integer(c_int), intent(in), value :: word_id, out_len
    real(c_float), intent(out) :: out_vec(out_len)
    integer(c_int) :: status
    integer :: fortran_id

    if (.not. g_initialized) then
      status = -1
      return
    end if

    fortran_id = word_id + 1
    if (fortran_id < 1 .or. fortran_id > g_vocab_size .or. out_len /= g_dim) then
      status = -2
      return
    end if

    out_vec(:) = g_w_in(:, fortran_id)
    status = 0
  end function wvec_get_embedding

  !> Request graceful shutdown (called from signal handler)
  subroutine wvec_shutdown_request() bind(C, name="wvec_shutdown_request")
    g_shutdown_requested = .true.
  end subroutine wvec_shutdown_request

  !> Check if shutdown was requested
  !> Returns: 1 if shutdown requested, 0 otherwise
  function wvec_shutdown_check() result(requested) bind(C, name="wvec_shutdown_check")
    integer(c_int) :: requested
    if (g_shutdown_requested) then
      requested = 1
    else
      requested = 0
    end if
  end function wvec_shutdown_check

  !> Reset shutdown flag (call before starting new training)
  subroutine wvec_shutdown_reset() bind(C, name="wvec_shutdown_reset")
    g_shutdown_requested = .false.
  end subroutine wvec_shutdown_reset

end module wvec_model
