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

end module wvec_checkpoint
