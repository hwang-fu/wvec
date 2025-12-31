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

  !> Train on a corpus of token IDs
  !> Uses OpenMP for parallel training (Hogwild style)
  function wvec_train_corpus(token_ids, n_tokens, window, n_neg, neg_table, neg_table_size, lr) &
    result(status) bind(C, name="wvec_train_corpus")
    integer(c_int), intent(in), value :: n_tokens, window, n_neg, neg_table_size
    integer(c_int), intent(in) :: token_ids(n_tokens)
    integer(c_int), intent(in) :: neg_table(neg_table_size)
    real(c_float), intent(in), value :: lr
    integer(c_int) :: status

    integer :: i, j, ctx_start, ctx_end, center_id, context_id
    integer :: neg_idx, k
    integer(c_int), allocatable :: neg_ids(:)

    if (.not. g_initialized) then
      status = -1
      return
    end if

    !$omp parallel private(i, j, ctx_start, ctx_end, center_id, context_id, neg_ids, neg_idx, k)
    allocate (neg_ids(n_neg))

    !$omp do schedule(dynamic, 1000)
    do i = 1, n_tokens
      center_id = token_ids(i)

      ! Context window bounds
      ctx_start = max(1, i - window)
      ctx_end = min(n_tokens, i + window)

      ! Train with each context word
      do j = ctx_start, ctx_end
        if (j == i) cycle  ! Skip center word itself
        context_id = token_ids(j)

        ! Sample negative words from table
        do k = 1, n_neg
          neg_idx = modulo(i * 7 + j * 13 + k * 17, neg_table_size) + 1
          neg_ids(k) = neg_table(neg_idx)
        end do

        ! Train this pair (updates shared g_w_in, g_w_out)
        call train_pair_internal(center_id, context_id, neg_ids, n_neg, lr)
      end do
    end do
    !$omp end do

    deallocate (neg_ids)
    !$omp end parallel

    status = 0
  end function wvec_train_corpus

  !> Internal training routine for skip-gram with negative sampling (not exported to C)
  !>
  !> Skip-gram objective: maximize P(context | center) while minimizing P(negative | center)
  !>
  !> Gradient formula (from original word2vec):
  !>   g = (label - σ(score)) × learning_rate
  !>
  !> where:
  !>   - score = dot(center_embedding, target_embedding)
  !>   - σ(x) = 1 / (1 + exp(-x))  (sigmoid function)
  !>   - label = 1 for positive (context) pairs
  !>   - label = 0 for negative samples
  !>
  !> Update rules:
  !>   - target_embedding += g × center_embedding
  !>   - center_embedding += g × target_embedding (accumulated, applied at end)
  !>
  !> Intuition:
  !>   - Positive pair with low  score -> large    g -> push vectors together
  !>   - Negative pair with high score -> negative g -> push vectors apart
  subroutine train_pair_internal(center_id, context_id, neg_ids, n_neg, lr)
    integer(c_int), intent(in) :: center_id, context_id, n_neg
    integer(c_int), intent(in) :: neg_ids(n_neg)
    real(c_float), intent(in) :: lr

    real(c_float) :: score, g
    real(c_float), allocatable :: grad_center(:)
    integer :: dim, i, neg_id_fortran, center_fortran, context_fortran
    integer :: one

    dim = g_dim
    one = 1  ! BLAS stride (contiguous memory access)

    ! Allocate buffer to accumulate gradients for center word.
    ! We apply all updates at once at the end for efficiency.
    allocate (grad_center(dim))
    grad_center = 0.0

    ! Convert from 0-indexed (C) to 1-indexed (Fortran)
    center_fortran = center_id + 1
    context_fortran = context_id + 1

    ! Positive sample (center, context): make these vectors similar
    ! label = 1, so g = (1 - σ(score)) x lr
    ! If score is already high (correct), σ ≈ 1, g ≈ 0 (small update)
    ! If score is low (wrong), σ ≈ 0, g ≈ lr (large update to push together)
    score = sdot(dim, g_w_in(1, center_fortran), one, g_w_out(1, context_fortran), one)
    g = (1.0 - sigmoid(score)) * lr
    call saxpy(dim, g, g_w_out(1, context_fortran), one, grad_center, one)  ! accumulate
    call saxpy(dim, g, g_w_in(1, center_fortran), one, g_w_out(1, context_fortran), one)  ! update context

    ! Negative samples: make these vectors dissimilar
    ! label = 0, so g = -σ(score) x lr
    ! If score is high (wrong), σ ≈ 1, g ≈ -lr (large negative update to push apart)
    ! If score is low (correct), σ ≈ 0, g ≈ 0 (small update)
    do i = 1, n_neg
      neg_id_fortran = neg_ids(i) + 1
      score = sdot(dim, g_w_in(1, center_fortran), one, g_w_out(1, neg_id_fortran), one)
      g = -sigmoid(score) * lr
      call saxpy(dim, g, g_w_out(1, neg_id_fortran), one, grad_center, one)  ! accumulate
      call saxpy(dim, g, g_w_in(1, center_fortran), one, g_w_out(1, neg_id_fortran), one)  ! update negative
    end do

    ! Apply accumulated gradient to center embedding
    call saxpy(dim, 1.0, grad_center, one, g_w_in(1, center_fortran), one)

    deallocate (grad_center)
  end subroutine train_pair_internal

end module wvec_train
