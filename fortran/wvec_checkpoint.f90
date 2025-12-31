! wvec_checkpoint.f90 - Save and restore training state
module wvec_checkpoint
  use, intrinsic :: iso_c_binding
  use wvec_types
  use wvec_model
  implicit none

  !> Magic bytes for checkpoint file: "WVCK" (Word Vec ChecKpoint)
  character(len=4), parameter :: CHECKPOINT_MAGIC = "WVCK"
  integer(c_int), parameter :: CHECKPOINT_VERSION = 1

contains

  !> Save model state to binary checkpoint file
  !> Parameters:
  !>   filename: null-terminated C string with file path
  !>   filename_len: length of filename (not including null)
  !>   epoch: current training epoch
  !>   learning_rate: current learning rate
  !> Returns: 0 on success, negative on error
  !>   -1: model not initialized
  !>   -4: file I/O error
  function wvec_checkpoint_save(filename, filename_len, epoch, learning_rate) &
    result(status) bind(C, name="wvec_checkpoint_save")
    character(kind=c_char), intent(in) :: filename(*)
    integer(c_int), intent(in), value :: filename_len
    integer(c_int), intent(in), value :: epoch
    real(c_float), intent(in), value :: learning_rate
    integer(c_int) :: status

    character(len=:), allocatable :: fpath
    integer :: unit_num, ios, i

    ! Check model is initialized
    if (.not. g_initialized) then
      status = -1  ! ERR_NOT_INITIALIZED
      return
    end if

    ! Convert C string to Fortran string
    allocate (character(len=filename_len) :: fpath)
    do i = 1, filename_len
      fpath(i:i) = filename(i)
    end do

    ! Open file for binary writing
    open (newunit=unit_num, file=fpath, status='replace', access='stream', &
          form='unformatted', iostat=ios)
    if (ios /= 0) then
      status = -4  ! ERR_FILE_IO
      deallocate (fpath)
      return
    end if

    ! Write header
    write (unit_num, iostat=ios) CHECKPOINT_MAGIC
    if (ios /= 0) goto 100
    write (unit_num, iostat=ios) CHECKPOINT_VERSION
    if (ios /= 0) goto 100
    write (unit_num, iostat=ios) g_vocab_size
    if (ios /= 0) goto 100
    write (unit_num, iostat=ios) g_dim
    if (ios /= 0) goto 100
    write (unit_num, iostat=ios) epoch
    if (ios /= 0) goto 100
    write (unit_num, iostat=ios) learning_rate
    if (ios /= 0) goto 100

    ! Write embedding matrices (column-major, as stored in Fortran)
    write (unit_num, iostat=ios) g_w_in
    if (ios /= 0) goto 100
    write (unit_num, iostat=ios) g_w_out
    if (ios /= 0) goto 100

    close (unit_num)
    deallocate (fpath)
    status = 0  ! SUCCESS
    return

    ! Error handler for write failures
100 close (unit_num)
    deallocate (fpath)
    status = -4  ! ERR_FILE_IO
  end function wvec_checkpoint_save

  function wvec_checkpoint_load(filename, filename_len, epoch, learning_rate) &
    result(status) bind(C, name="wvec_checkpoint_load")
    character(kind=c_char), intent(in) :: filename(*)
    integer(c_int), intent(in), value :: filename_len
    integer(c_int), intent(out) :: epoch
    real(c_float), intent(out) :: learning_rate
    integer(c_int) :: status

    character(len=:), allocatable :: fpath
    character(len=4) :: magic
    integer(c_int) :: version, vocab_size, dim
    integer :: unit_num, ios, i, alloc_stat

    ! Convert C string to Fortran string
    allocate (character(len=filename_len) :: fpath)
    do i = 1, filename_len
      fpath(i:i) = filename(i)
    end do

    ! Open file for binary reading
    open (newunit=unit_num, file=fpath, status='old', access='stream', &
          form='unformatted', iostat=ios)
    if (ios /= 0) then
      status = -4  ! ERR_FILE_IO
      deallocate (fpath)
      return
    end if

    ! Read and validate header
    read (unit_num, iostat=ios) magic
    if (ios /= 0) goto 200
    if (magic /= CHECKPOINT_MAGIC) then
      close (unit_num)
      deallocate (fpath)
      status = -5  ! ERR_INVALID_MAGIC
      return
    end if

    read (unit_num, iostat=ios) version
    if (ios /= 0) goto 200
    if (version /= CHECKPOINT_VERSION) then
      close (unit_num)
      deallocate (fpath)
      status = -6  ! ERR_UNSUPPORTED_VERSION
      return
    end if

    read (unit_num, iostat=ios) vocab_size
    if (ios /= 0) goto 200
    read (unit_num, iostat=ios) dim
    if (ios /= 0) goto 200
    read (unit_num, iostat=ios) epoch
    if (ios /= 0) goto 200
    read (unit_num, iostat=ios) learning_rate
    if (ios /= 0) goto 200

    ! Free existing model and set dimensions
    call wvec_model_free()
    g_vocab_size = vocab_size
    g_dim = dim

    ! Allocate embedding matrices
    allocate (g_w_in(dim, vocab_size), stat=alloc_stat)
    if (alloc_stat /= 0) then
      close (unit_num)
      deallocate (fpath)
      status = -3  ! ERR_OUT_OF_MEMORY
      return
    end if

    allocate (g_w_out(dim, vocab_size), stat=alloc_stat)
    if (alloc_stat /= 0) then
      deallocate (g_w_in)
      close (unit_num)
      deallocate (fpath)
      status = -3  ! ERR_OUT_OF_MEMORY
      return
    end if

    ! Read embedding matrices
    read (unit_num, iostat=ios) g_w_in
    if (ios /= 0) goto 210
    read (unit_num, iostat=ios) g_w_out
    if (ios /= 0) goto 210

    close (unit_num)
    deallocate (fpath)
    g_initialized = .true.
    status = 0  ! SUCCESS
    return

    ! Error handler for read failures (before allocation)
200 close (unit_num)
    deallocate (fpath)
    status = -4  ! ERR_FILE_IO
    return

    ! Error handler for read failures (after allocation)
210 if (allocated(g_w_in)) deallocate (g_w_in)
    if (allocated(g_w_out)) deallocate (g_w_out)
    g_vocab_size = 0
    g_dim = 0
    close (unit_num)
    deallocate (fpath)
    status = -4  ! ERR_FILE_IO
  end function wvec_checkpoint_load

end module wvec_checkpoint
