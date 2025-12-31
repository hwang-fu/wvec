! wvec_thermal.f90 - CPU temperature monitoring
module wvec_thermal
  use, intrinsic :: iso_c_binding
  implicit none

  !> Default thermal zone path (x86_pkg_temp is reliable for Intel CPUs)
  character(len=*), parameter :: DEFAULT_THERMAL_PATH = &
                                 "/sys/class/thermal/thermal_zone10/temp"

  !> Default temperature threshold in Celsius
  integer, parameter :: DEFAULT_THRESHOLD_C = 85

contains

  !> Read CPU temperature from sysfs
  !> Parameters:
  !>   path: path to thermal zone temp file (C string)
  !>   path_len: length of path string
  !>   temp_mc: output temperature in millidegrees Celsius
  !> Returns: 0 on success, -4 on file I/O error
  function wvec_thermal_read(path, path_len, temp_mc) &
    result(status) bind(C, name="wvec_thermal_read")
    character(kind=c_char), intent(in) :: path(*)
    integer(c_int), intent(in), value :: path_len
    integer(c_int), intent(out) :: temp_mc
    integer(c_int) :: status

    character(len=:), allocatable :: fpath
    integer :: unit_num, ios, i

    ! Convert C string to Fortran string
    allocate (character(len=path_len) :: fpath)
    do i = 1, path_len
      fpath(i:i) = path(i)
    end do

    ! Open and read temperature file
    open (newunit=unit_num, file=fpath, status='old', action='read', iostat=ios)
    if (ios /= 0) then
      deallocate (fpath)
      temp_mc = 0
      status = -4  ! ERR_FILE_IO
      return
    end if

    read (unit_num, *, iostat=ios) temp_mc
    close (unit_num)
    deallocate (fpath)

    if (ios /= 0) then
      temp_mc = 0
      status = -4  ! ERR_FILE_IO
      return
    end if

    status = 0  ! SUCCESS
  end function wvec_thermal_read

  !> Check if CPU is overheating
  !> Parameters:
  !>   path: path to thermal zone temp file (C string)
  !>   path_len: length of path string
  !>   threshold_c: temperature threshold in Celsius
  !> Returns: 1 if overheating, 0 if OK, negative on error
  function wvec_thermal_check(path, path_len, threshold_c) &
    result(is_hot) bind(C, name="wvec_thermal_check")
    character(kind=c_char), intent(in) :: path(*)
    integer(c_int), intent(in), value :: path_len
    integer(c_int), intent(in), value :: threshold_c
    integer(c_int) :: is_hot

    integer(c_int) :: temp_mc, status, threshold_mc

    status = wvec_thermal_read(path, path_len, temp_mc)
    if (status /= 0) then
      is_hot = status  ! Return error code
      return
    end if

    ! Convert threshold to millidegrees
    threshold_mc = threshold_c * 1000

    if (temp_mc >= threshold_mc) then
      is_hot = 1  ! Overheating
    else
      is_hot = 0  ! OK
    end if
  end function wvec_thermal_check

  !> Get current temperature in Celsius (convenience function)
  !> Parameters:
  !>   path: path to thermal zone temp file (C string)
  !>   path_len: length of path string
  !>   temp_c: output temperature in Celsius
  !> Returns: 0 on success, negative on error
  function wvec_thermal_get_celsius(path, path_len, temp_c) &
    result(status) bind(C, name="wvec_thermal_get_celsius")
    character(kind=c_char), intent(in) :: path(*)
    integer(c_int), intent(in), value :: path_len
    integer(c_int), intent(out) :: temp_c
    integer(c_int) :: status

    integer(c_int) :: temp_mc

    status = wvec_thermal_read(path, path_len, temp_mc)
    if (status /= 0) then
      temp_c = 0
      return
    end if

    temp_c = temp_mc / 1000
  end function wvec_thermal_get_celsius

end module wvec_thermal
