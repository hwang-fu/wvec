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

end module wvec_thermal
