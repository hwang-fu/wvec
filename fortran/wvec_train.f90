! wvec_train.f90 - Skip-gram training with negative sampling
module wvec_train
  use, intrinsic :: iso_c_binding
  use wvec_types
  use wvec_blas
  use wvec_model
  implicit none

contains

  !> Sigmoid function: σ(x) = 1 / (1 + exp(-x))
  pure real(c_float) function sigmoid(x)
    real(c_float), intent(in) :: x
    real(c_float) :: clamped

    clamped = max(-20.0, min(20.0, x))
    sigmoid = 1.0 / (1.0 + exp(-clamped))
  end function sigmoid

  !> Train one skip-gram pair with negative sampling
  !>
  !> Uses the gradient formulation from original word2vec:
  !>   g = (label - sigmoid(score)) * learning_rate
  !> where label=1 for positive pairs, label=0 for negative pairs
  function wvec_train_pair(center_id, context_id, neg_ids, n_neg, lr) &
    result(status) bind(C, name="wvec_train_pair")
    integer(c_int), intent(in), value :: center_id, context_id, n_neg
    integer(c_int), intent(in) :: neg_ids(n_neg)
    real(c_float), intent(in), value :: lr
    integer(c_int) :: status

    real(c_float) :: score, g
    real(c_float), allocatable :: grad_center(:)
    integer :: dim, i, neg_id_fortran, center_fortran, context_fortran
    integer :: one

    if (.not. g_initialized) then
      status = -1
      return
    end if

    dim = g_dim
    one = 1
    allocate (grad_center(dim))
    grad_center = 0.0

    ! Convert to 1-indexed
    center_fortran = center_id + 1
    context_fortran = context_id + 1

    ! --- Positive sample (center, context), label = 1 ---
    score = sdot(dim, g_w_in(1, center_fortran), one, &
                 g_w_out(1, context_fortran), one)
    g = (1.0 - sigmoid(score)) * lr

    ! Accumulate gradient for center
    call saxpy(dim, g, g_w_out(1, context_fortran), one, grad_center, one)
    ! Update context embedding
    call saxpy(dim, g, g_w_in(1, center_fortran), one, &
               g_w_out(1, context_fortran), one)

    ! --- Negative samples, label = 0 ---
    do i = 1, n_neg
      neg_id_fortran = neg_ids(i) + 1

      score = sdot(dim, g_w_in(1, center_fortran), one, &
                   g_w_out(1, neg_id_fortran), one)
      g = (0.0 - sigmoid(score)) * lr  ! = -sigmoid(score) * lr

      ! Accumulate gradient for center
      call saxpy(dim, g, g_w_out(1, neg_id_fortran), one, grad_center, one)
      ! Update negative embedding
      call saxpy(dim, g, g_w_in(1, center_fortran), one, &
                 g_w_out(1, neg_id_fortran), one)
    end do

    ! --- Update center embedding ---
    call saxpy(dim, 1.0, grad_center, one, g_w_in(1, center_fortran), one)

    deallocate (grad_center)
    status = 0
  end function wvec_train_pair

end module wvec_train
