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

end module wvec_checkpoint
