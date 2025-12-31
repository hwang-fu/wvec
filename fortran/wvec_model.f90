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

end module wvec_model
